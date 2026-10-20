//! M5 feasibility probe: original generated artwork, completed GPU timings, CPU pixel reference.
//! `cargo run --release -p vectorcraft-render --features gpu-probe --example gpu_probe -- 50000 /tmp/gpu-probe`
#![forbid(unsafe_code)]

use std::time::Instant;

use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node};
use vectorcraft_geom::{Affine, Point, Rect, shapes};
use vectorcraft_render::{RenderOptions, Rendered, Renderer};

fn main() -> Result<(), String> {
    pollster::block_on(run())
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 1_000_000) as f64 / 1_000_000.0
    }
}

fn artwork(n: usize) -> Result<Document, String> {
    let mut doc = Document::new(1600.0, 1200.0);
    let layer = doc.layers.first().map(|n| n.id);
    let mut rng = Rng(42);
    for i in 0..n {
        let (x, y, s) = (rng.next() * 1600.0, rng.next() * 1200.0, 3.0 + rng.next() * 22.0);
        let color = Color::rgb(rng.next() as f32, rng.next() as f32, rng.next() as f32);
        let path = match i % 3 {
            0 => shapes::ellipse(Rect::from_center_size(Point::new(x, y), (2.0 * s, 2.0 * s))),
            1 => shapes::rectangle(Rect::new(x, y, x + 2.0 * s, y + s)),
            _ => shapes::star(Point::new(x, y), s, s / 2.0, 5, 0.0),
        };
        let opacity = if i % 3 == 1 { 0.8 } else { 1.0 };
        let id = doc.alloc_id();
        let mut node = Node::path(id, path, Appearance::basic(Paint::solid(color), Paint::None, 0.0));
        node.opacity = opacity;
        doc.insert(layer, usize::MAX, node).map_err(|e| e.to_string())?;
    }
    Ok(doc)
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--file") {
        return file_probe(args.get(1..).unwrap_or_default()).await;
    }
    let n = std::env::args().nth(1).map(|s| s.parse::<usize>()).transpose().map_err(|e| e.to_string())?.unwrap_or(50_000);
    if !(1..=100_000).contains(&n) {
        return Err("path count must be between 1 and 100000".into());
    }
    let output = std::env::args().nth(2).unwrap_or_else(|| "/tmp/vectorcraft-gpu-probe".into());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.map_err(|e| e.to_string())?;
    let info = adapter.get_info();
    println!("adapter={} backend={:?} kind={:?}", info.name, info.backend, info.device_type);
    println!(
        "workload=solid_fills_with_object_opacity cpu_threads={} logical_cores={}",
        vectorcraft_render::default_threads(),
        std::thread::available_parallelism().map_or(1, usize::from)
    );
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.map_err(|e| e.to_string())?;
    let (width, height) = (2880u16, 1800u16);
    let t = Instant::now();
    let doc = artwork(n)?;
    println!("paths={n} size={width}x{height} document_ms={:.2}", t.elapsed().as_secs_f64() * 1000.0);
    let transform = Affine::scale_non_uniform(f64::from(width) / 1600.0, f64::from(height) / 1200.0);
    let mut cpu = Renderer::new();
    let mut cpu_ms = Vec::new();
    let mut reference = None;
    for _ in 0..6 {
        let t = Instant::now();
        reference = Some(cpu.render(&doc, width.into(), height.into(), transform, &RenderOptions::default()));
        cpu_ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    println!("cpu_cold_ms={:.2} cpu_warm_median_ms={:.2}", cpu_ms.first().copied().unwrap_or_default(), median(cpu_ms.get(1..).unwrap_or_default()));
    let reference = reference.ok_or("no CPU reference")?;
    std::fs::write(format!("{output}-cpu.png"), reference.to_png()?).map_err(|e| e.to_string())?;
    compute_probe(&device, &queue, &doc, &reference, transform, &output, &RenderOptions::default())?;
    // Integer edges make this a strict compositing/alpha/stroke check independent of curve AA.
    // Large world coordinates also exercise f64 view preparation before GPU f32 encoding.
    let mut fixture = Document::new(256.0, 256.0);
    let origin = 1_000_000_000.0;
    for (index, rect) in [Rect::new(20.0, 20.0, 120.0, 120.0), Rect::new(80.0, 80.0, 220.0, 220.0)].into_iter().enumerate() {
        let id = fixture.alloc_id();
        let rect = rect + vectorcraft_geom::Vec2::new(origin, origin);
        let mut node =
            Node::path(id, shapes::rectangle(rect), Appearance::basic(Paint::solid(Color::rgb(0.8, 0.2, 0.4)), Paint::solid(Color::BLACK), 10.0));
        node.opacity = if index == 0 { 0.6 } else { 0.8 };
        for item in &mut node.appearance.items {
            if let vectorcraft_doc::AppearanceItem::Stroke(stroke) = item {
                stroke.opacity = 0.5;
            }
        }
        fixture.insert(None, usize::MAX, node).map_err(|e| e.to_string())?;
    }
    let view = Affine::translate((-origin, -origin));
    let reference = cpu.render(&fixture, 256, 256, view, &RenderOptions::default());
    let max = compute_probe(&device, &queue, &fixture, &reference, view, &format!("{output}-conformance"), &RenderOptions::default())?;
    if max > 4 {
        return Err(format!("GPU compositing/stroke conformance error {max} exceeds 4/255"));
    }
    Ok(())
}

fn compute_probe(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    doc: &Document,
    reference: &Rendered,
    transform: Affine,
    output: &str,
    opts: &RenderOptions,
) -> Result<u8, String> {
    // This comparison tool always allocates all three targets, even for an RGB fixture.
    vectorcraft_render::gpu::CanvasLimits::for_device(device).check(reference.width, reference.height, true)?;
    let descriptor = wgpu::TextureDescriptor {
        label: Some("compute candidate"),
        size: wgpu::Extent3d { width: reference.width, height: reference.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&descriptor);
    let converted = device.create_texture(&descriptor);
    let black = device.create_texture(&descriptor);
    let black_view = black.create_view(&Default::default());
    let view = texture.create_view(&Default::default());
    let mut alpha = vectorcraft_render::gpu::CompositePass::new(device);
    let mut renderer = compute_vello::Renderer::new(
        device,
        compute_vello::RendererOptions {
            antialiasing_support: compute_vello::AaSupport::area_only(),
            num_init_threads: std::num::NonZeroUsize::new(1),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())?;
    let mut preparation = Renderer::new();
    let mut preparation_ms = Vec::new();
    let mut rendering_ms = Vec::new();
    for _ in 0..10 {
        let t = Instant::now();
        let prepared = preparation.prepare_gpu_with_limits(
            doc,
            reference.width,
            reference.height,
            transform,
            opts,
            vectorcraft_render::gpu::CanvasLimits::for_device(device),
        )?;
        if preparation_ms.is_empty() {
            println!(
                "prepared paths={} clips={} images={} cmyk={} open_clips={}",
                prepared.scene.encoding().n_paths,
                prepared.scene.encoding().n_clips,
                prepared.images.len(),
                prepared.cmyk.is_some(),
                prepared.scene.encoding().n_open_clips
            );
        }
        preparation_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        let t = Instant::now();
        renderer
            .render_to_texture(
                device,
                queue,
                &prepared.scene,
                &view,
                &compute_vello::RenderParams {
                    base_color: compute_vello::peniko::Color::TRANSPARENT,
                    width: reference.width,
                    height: reference.height,
                    antialiasing_method: compute_vello::AaConfig::Area,
                },
            )
            .map_err(|e| e.to_string())?;
        if let Some(cmyk) = &prepared.cmyk {
            renderer
                .render_to_texture(
                    device,
                    queue,
                    &cmyk.black,
                    &black_view,
                    &compute_vello::RenderParams {
                        base_color: compute_vello::peniko::Color::TRANSPARENT,
                        width: reference.width,
                        height: reference.height,
                        antialiasing_method: compute_vello::AaConfig::Area,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        let alpha_group = alpha.bind(device, &view, &black_view, &converted.create_view(&Default::default()), prepared.cmyk.as_ref());
        let mut encoder = device.create_command_encoder(&Default::default());
        alpha.encode(&mut encoder, &alpha_group, prepared.width, prepared.height);
        queue.submit([encoder.finish()]);
        device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| e.to_string())?;
        rendering_ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let totals: Vec<f64> = preparation_ms.iter().zip(&rendering_ms).map(|(a, b)| a + b).collect();
    println!(
        "compute_prepare_warm_median_ms={:.2} compute_gpu_warm_median_ms={:.2} compute_total_warm_median_ms={:.2}",
        median(preparation_ms.get(1..).unwrap_or_default()),
        median(rendering_ms.get(1..).unwrap_or_default()),
        median(totals.get(1..).unwrap_or_default())
    );
    let gpu = readback(device, queue, &converted)?;
    let max = reference.pixels.iter().zip(&gpu.pixels).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or_default();
    let total: u64 = reference.pixels.iter().zip(&gpu.pixels).map(|(a, b)| u64::from(a.abs_diff(*b))).sum();
    println!("compute_pixel_mean_abs={:.4} compute_max_channel_error={max}", total as f64 / reference.pixels.len() as f64);
    std::fs::write(format!("{output}-compute.png"), gpu.to_png()?).map_err(|e| e.to_string())?;
    Ok(max)
}

fn median(values: &[f64]) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or_default()
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Result<Rendered, String> {
    let stride = (texture.width() * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("probe readback"),
        size: u64::from(stride) * u64::from(texture.height()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(stride), rows_per_image: None },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        // The receiver may have gone away after a device error.
        let _ = tx.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    let mut pixels = Vec::new();
    {
        let mapped = buffer.slice(..).get_mapped_range().map_err(|e| e.to_string())?;
        for row in mapped.chunks_exact(stride as usize) {
            pixels.extend_from_slice(row.get(..texture.width() as usize * 4).ok_or("short readback row")?);
        }
    }
    buffer.unmap();
    Ok(Rendered { width: texture.width(), height: texture.height(), pixels })
}

async fn file_probe(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or("--file needs an input path")?;
    let output = args.get(1).map_or("/tmp/vectorcraft-gpu-file", String::as_str);
    let mut session = vectorcraft_testkit::Session::new();
    session.execute("document.open", &vectorcraft_testkit::json!({"path": path})).map_err(|e| e.to_string())?;
    let doc = session.doc().map_err(|e| e.to_string())?.doc.clone();
    println!("file={} mode={:?} page_isolate={} page_knockout={}", path, doc.color_mode, doc.page_isolate, doc.page_knockout);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.map_err(|e| e.to_string())?;
    println!("adapter={:?}", adapter.get_info());
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.map_err(|e| e.to_string())?;
    let custom_view = args.len() > 2;
    let (w, h, center, z) = if custom_view {
        if args.len() != 8 || args.get(2).map(String::as_str) != Some("--view") {
            return Err("expected --file INPUT OUTPUT [--view WIDTH HEIGHT CENTER_X CENTER_Y PIXELS_PER_POINT]".into());
        }
        let dimension = |i: usize| args.get(i).ok_or("missing view dimension")?.parse::<u32>().map_err(|e| e.to_string());
        let number = |i: usize| args.get(i).ok_or("missing view value")?.parse::<f64>().map_err(|e| e.to_string());
        let (x, y, z) = (number(5)?, number(6)?, number(7)?);
        if ![x, y, z].iter().all(|v| v.is_finite()) || x.abs().max(y.abs()) > 1e9 || !(1e-4..=1e4).contains(&z) {
            return Err("view center and scale must be finite and within range".into());
        }
        (dimension(3)?, dimension(4)?, Point::new(x, y), z)
    } else {
        let b = doc.artboards.first().ok_or("no artboard")?.rect;
        (3840, 2160, b.center(), (3840.0 / b.width()).min(2160.0 / b.height()) * 0.95)
    };
    vectorcraft_render::gpu::CanvasLimits::for_device(&device).check(w, h, true)?;
    let view = Affine::translate((f64::from(w) / 2.0, f64::from(h) / 2.0)) * Affine::scale(z) * Affine::translate(-center.to_vec2());
    println!("view={w}x{h} center={center:?} scale={z}");
    let opts = RenderOptions { image_mips: true, artboards: !custom_view, ..Default::default() };
    let mut cpu = Renderer::new();
    let mut timings = Vec::new();
    let mut reference = None;
    for _ in 0..6 {
        let started = Instant::now();
        reference = Some(cpu.render(&doc, w, h, view, &opts));
        timings.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let reference = reference.ok_or("no CPU reference")?;
    println!(
        "cpu_cold_ms={:.2} cpu_warm_median_ms={:.2} ink_threads={}",
        timings.first().copied().unwrap_or_default(),
        median(timings.get(1..).unwrap_or_default()),
        cpu.stats.ink_threads
    );
    std::fs::write(format!("{output}-cpu.png"), reference.to_png()?).map_err(|e| e.to_string())?;
    compute_probe(&device, &queue, &doc, &reference, view, output, &opts)?;
    Ok(())
}
