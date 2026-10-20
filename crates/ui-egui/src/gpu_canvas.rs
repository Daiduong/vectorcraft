//! Optional canvas texture, rendered on the host's existing device and queue.
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::task::{Context, Poll, Waker};

use egui_wgpu::{RenderState, wgpu};
use vectorcraft_render::{
    gpu::{AtlasTracker, CanvasLimits, CompositePass, PreparedScene},
    gpu_backend,
};

type ErrorFuture = Pin<Box<dyn Future<Output = Option<wgpu::Error>>>>;

struct Target {
    width: u16,
    height: u16,
    texture: wgpu::Texture,
    source: wgpu::TextureView,
    black: Option<wgpu::TextureView>,
}

pub(crate) struct GpuCanvas {
    state: RenderState,
    target: Option<Target>,
    renderer: Option<gpu_backend::Renderer>,
    alpha: Option<CompositePass>,
    atlas: AtlasTracker,
    id: Option<egui::TextureId>,
    errors: Vec<ErrorFuture>,
    in_flight: Arc<AtomicUsize>,
    pub(crate) disabled: Option<String>,
}

impl GpuCanvas {
    fn new(state: RenderState) -> Self {
        Self {
            state,
            target: None,
            renderer: None,
            alpha: None,
            atlas: Default::default(),
            id: None,
            errors: Vec::new(),
            in_flight: Arc::new(AtomicUsize::new(0)),
            disabled: None,
        }
    }

    pub(crate) fn poll(&mut self) {
        if let Err(error) = self.state.device.poll(wgpu::PollType::Poll) {
            self.disabled = Some(format!("GPU canvas device: {error}"));
        }
        let mut context = Context::from_waker(Waker::noop());
        let mut pending = Vec::new();
        for mut future in self.errors.drain(..) {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(Some(error)) => self.disabled = Some(format!("GPU canvas: {error}")),
                Poll::Ready(None) => {}
                Poll::Pending => pending.push(future),
            }
        }
        self.errors = pending;
    }

    pub(crate) fn pending(&self) -> bool {
        !self.errors.is_empty() || self.in_flight.load(Ordering::Acquire) > 0
    }

    pub(crate) fn limits(&self) -> CanvasLimits {
        CanvasLimits::for_device(&self.state.device)
    }

    /// Renders `prepared` into the canvas texture. `pixel`: Pixel Preview's document pixels,
    /// magnified hard-edged as the CPU canvas shows them.
    pub(crate) fn present(&mut self, prepared: PreparedScene, pixel: bool) -> Result<egui::TextureId, String> {
        self.poll();
        if let Some(reason) = &self.disabled {
            return Err(reason.clone());
        }
        if self.errors.len() >= 4 {
            return Err("GPU canvas validation has not completed".into());
        }
        if self.in_flight.load(Ordering::Acquire) >= 2 {
            return Err("GPU canvas frame queue limit reached".into());
        }
        let device = &self.state.device;
        let queue = &self.state.queue;
        let (width, height) = (prepared.width, prepared.height);
        self.limits().check(width.into(), height.into(), prepared.cmyk.is_some())?;
        if device.limits().max_compute_invocations_per_workgroup < 256 {
            let reason = "this graphics device has no compatible compute renderer".to_owned();
            self.disabled = Some(reason.clone());
            return Err(reason);
        }
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let allocation = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let result: Result<egui::TextureId, String> = (|| {
            if self.atlas.prepare(&prepared)? {
                // Reset only for actual allocator pressure or the retained-image budget. A pan
                // that reveals another image normally keeps pipelines and uploaded images.
                self.renderer = None;
            }
            if self.renderer.is_none() {
                self.renderer = Some(
                    gpu_backend::Renderer::new(
                        device,
                        gpu_backend::RendererOptions {
                            antialiasing_support: gpu_backend::AaSupport::area_only(),
                            num_init_threads: std::num::NonZeroUsize::new(1),
                            ..Default::default()
                        },
                    )
                    .map_err(|e| e.to_string())?,
                );
            }
            if self.alpha.is_none() {
                self.alpha = Some(CompositePass::new(device));
            }
            let alpha = self.alpha.as_mut().ok_or("GPU alpha pipeline is unavailable")?;
            if self.target.as_ref().is_none_or(|t| (t.width, t.height, t.black.is_some()) != (width, height, prepared.cmyk.is_some())) {
                let descriptor = wgpu::TextureDescriptor {
                    label: Some("VectorCraft GPU canvas"),
                    size: wgpu::Extent3d { width: width.into(), height: height.into(), depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                };
                let texture = device.create_texture(&descriptor);
                let source = device.create_texture(&descriptor).create_view(&Default::default());
                let black = prepared.cmyk.as_ref().map(|_| device.create_texture(&descriptor).create_view(&Default::default()));
                self.target = Some(Target { width, height, texture, source, black });
            }
            let target = self.target.as_ref().ok_or("GPU target was not created")?;
            let view = target.texture.create_view(&Default::default());
            let renderer = self.renderer.as_mut().ok_or("GPU renderer is unavailable")?;
            let params = gpu_backend::RenderParams {
                base_color: gpu_backend::peniko::Color::TRANSPARENT,
                width: width.into(),
                height: height.into(),
                antialiasing_method: gpu_backend::AaConfig::Area,
            };
            renderer.render_to_texture(device, queue, &prepared.scene, &target.source, &params).map_err(|e| e.to_string())?;
            if let Some(cmyk) = &prepared.cmyk
                && let Some(black) = &target.black
            {
                renderer.render_to_texture(device, queue, &cmyk.black, black, &params).map_err(|e| e.to_string())?;
            }
            let alpha_group = alpha.bind(device, &target.source, target.black.as_ref().unwrap_or(&target.source), &view, prepared.cmyk.as_ref());
            let mut encoder = device.create_command_encoder(&Default::default());
            alpha.encode(&mut encoder, &alpha_group, width, height);
            self.in_flight.fetch_add(1, Ordering::Release);
            queue.submit([encoder.finish()]);
            let in_flight = self.in_flight.clone();
            queue.on_submitted_work_done(move || {
                in_flight.fetch_sub(1, Ordering::Release);
            });
            let filter = if pixel { wgpu::FilterMode::Nearest } else { wgpu::FilterMode::Linear };
            let mut renderer = self.state.renderer.write();
            let id = match self.id {
                Some(id) => {
                    renderer.update_egui_texture_from_wgpu_texture(device, &view, filter, id);
                    id
                }
                None => renderer.register_native_texture(device, &view, filter),
            };
            self.id = Some(id);
            Ok(id)
        })();
        self.errors.push(Box::pin(allocation.pop()));
        self.errors.push(Box::pin(validation.pop()));
        if let Err(reason) = &result {
            self.disabled = Some(reason.clone());
        }
        result
    }
}

impl Drop for GpuCanvas {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.state.renderer.write().free_texture(&id);
        }
    }
}

impl crate::VectorcraftApp {
    /// Attaching the host's device does not initialize or select GPU rasterization.
    pub fn enable_gpu_canvas(&mut self, state: RenderState) {
        self.gpu_canvas = Some(GpuCanvas::new(state));
    }
}
