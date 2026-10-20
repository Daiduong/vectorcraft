//! Canvas output conversion. RGB changes alpha representation; CMYK composites the same two
//! ink planes and 17^4 XYZ table as the CPU, without GPU-to-CPU readback.
use super::CmykComposition;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

pub struct CompositePass {
    pipeline: wgpu::ComputePipeline,
    table: wgpu::Buffer,
    generation: Option<u64>,
}

pub(super) const COMPOSITE_SHADER: &str = r#"
struct Settings {
 flags: vec4<u32>,
 bg_cmy: vec4<f32>,
 bg_k: vec4<f32>,
 bg_rgb: vec4<f32>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var black: texture_2d<f32>;
@group(0) @binding(2) var output_image: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<storage, read> table: array<vec4<f32>>;
@group(0) @binding(4) var<uniform> settings: Settings;

fn encoded(v: f32) -> f32 {
 let x = clamp(v, 0.0, 1.0);
 var e = 12.92 * x;
 if x > 0.0031308 { e = 1.055 * pow(x, 1.0 / 2.4) - 0.055; }
 if e > 0.99999 { return 1.0; }
 if e < 0.000001 { return 0.0; }
 return e;
}
fn lookup(inks: vec4<f32>) -> vec3<f32> {
 let p = clamp(inks, vec4<f32>(0.0), vec4<f32>(1.0)) * 16.0;
 let lo = min(vec4<u32>(p), vec4<u32>(15u));
 let t = p - vec4<f32>(lo);
 var xyz = vec3<f32>(0.0);
 for (var corner = 0u; corner < 16u; corner++) {
  var weight = 1.0;
  var at = lo;
  for (var d = 0u; d < 4u; d++) {
   if ((corner >> d) & 1u) != 0u { weight *= t[d]; at[d] += 1u; }
   else { weight *= 1.0 - t[d]; }
  }
  if weight > 0.0 { xyz += weight * table[at.x * 4913u + at.y * 289u + at.z * 17u + at.w].xyz; }
 }
 return vec3<f32>(
  encoded(dot(vec3<f32>(3.1338561, -1.6168667, -0.4906146), xyz)),
  encoded(dot(vec3<f32>(-0.9787684, 1.9161415, 0.033454), xyz)),
  encoded(dot(vec3<f32>(0.0719453, -0.2289914, 1.4052427), xyz)));
}
@compute @workgroup_size(16, 16)
fn composite(@builtin(global_invocation_id) id: vec3<u32>) {
 let size = textureDimensions(output_image);
 if id.x >= size.x || id.y >= size.y { return; }
 let at = vec2<i32>(id.xy);
 let color = textureLoad(source, at, 0);
 if settings.flags.x == 0u {
  textureStore(output_image, at, vec4<f32>(color.rgb * color.a, color.a));
  return;
 }
 // Match the CPU's intermediate premultiplied RGBA8 rounding before unpremultiplication.
 let p = round(vec4<f32>(color.rgb * color.a, color.a) * 255.0) / 255.0;
 let kb = textureLoad(black, at, 0);
 let q = round(vec4<f32>(kb.rgb * kb.a, kb.a) * 255.0) / 255.0;
 if p.a == 0.0 { textureStore(output_image, at, vec4<f32>(0.0)); return; }
 if settings.flags.z != 0u && all(round(p * 255.0) == round(settings.bg_cmy * 255.0)) && all(round(q * 255.0) == round(settings.bg_k * 255.0)) {
  textureStore(output_image, at, settings.bg_rgb); return;
 }
 let cmy = min(p.rgb / p.a, vec3<f32>(1.0));
 var k = 1.0;
 if q.a > 0.0 { k = min(q.r / q.a, 1.0); }
 let inks = vec4<f32>(vec3<f32>(1.0) - cmy, 1.0 - k);
 var rgb = vec3<f32>(0.0);
 if !(settings.flags.y != 0u && all(inks.xyz <= vec3<f32>(0.0)) && inks.w >= 1.0) { rgb = lookup(inks); }
 textureStore(output_image, at, vec4<f32>(rgb * p.a, p.a));
}
"#;

impl CompositePass {
    /// The host captures validation and allocation errors around initialization and submission.
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("VectorCraft canvas color and alpha"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPOSITE_SHADER)),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("VectorCraft canvas color and alpha"),
            layout: None,
            module: &shader,
            entry_point: Some("composite"),
            compilation_options: Default::default(),
            cache: None,
        });
        let table = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VectorCraft RGB lookup placeholder"),
            contents: &[0; 16],
            usage: wgpu::BufferUsages::STORAGE,
        });
        Self { pipeline, table, generation: None }
    }

    pub fn bind(
        &mut self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        black: &wgpu::TextureView,
        target: &wgpu::TextureView,
        cmyk: Option<&CmykComposition>,
    ) -> wgpu::BindGroup {
        let mut settings = Vec::with_capacity(64);
        for flag in
            [u32::from(cmyk.is_some()), u32::from(cmyk.is_some_and(|c| c.rich_black)), u32::from(cmyk.is_some_and(|c| c.background.is_some())), 0]
        {
            settings.extend_from_slice(&flag.to_le_bytes());
        }
        let bg = cmyk.and_then(|c| c.background).unwrap_or([[0; 4]; 3]);
        for color in bg {
            for component in color {
                settings.extend_from_slice(&(f32::from(component) / 255.0).to_le_bytes());
            }
        }
        if let Some(cmyk) = cmyk
            && self.generation != Some(cmyk.generation)
        {
            let bytes: Vec<u8> = cmyk.lookup.iter().flat_map(|row| row.iter().flat_map(|v| v.to_le_bytes())).collect();
            self.table = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("VectorCraft CMYK XYZ display table"),
                contents: &bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });
            self.generation = Some(cmyk.generation);
        }
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VectorCraft canvas color settings"),
            contents: &settings,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VectorCraft canvas output views"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(source) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(black) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(target) },
                wgpu::BindGroupEntry { binding: 3, resource: self.table.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: params.as_entire_binding() },
            ],
        })
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, group: &wgpu::BindGroup, width: u16, height: u16) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("VectorCraft canvas color and alpha"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(u32::from(width).div_ceil(16), u32::from(height).div_ceil(16), 1);
    }
}
