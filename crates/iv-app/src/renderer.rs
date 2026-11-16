//! `wgpu` opaque swapchain presenter with 16K device limit request, tiled sub-textures, and GPU blur/unsharp shaders.

use bytemuck::{Pod, Zeroable};
use iv_core::layout::{ViewportLayoutCalculator, ViewportState};
use iv_core::settings::ViewerBackground;
use iv_media::resize_filter::FilterUniforms;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ShaderParams {
    tex_size: [f32; 2],
    viewport_size: [f32; 2],
    scale: [f32; 2],
    offset: [f32; 2],
    blur_sigma: f32,
    sharpen_sigma: f32,
    sharpen_amount: f32,
    checker_mode: f32,
    rot_turns: f32,
    flip_h: f32,
    flip_v: f32,
    _pad: f32,
}

pub struct GpuTile {
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

pub struct WindowGpuSurface {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub needs_reconfigure: bool,
    pub egui_renderer: egui_wgpu::Renderer,
}

pub struct GpuRenderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub max_texture_dim: u32,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
}

const IMAGE_SHADER_WGSL: &str = r#"
struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    tex_size: vec2<f32>,
    viewport_size: vec2<f32>,
    scale: vec2<f32>,
    offset: vec2<f32>,
    blur_sigma: f32,
    sharpen_sigma: f32,
    sharpen_amount: f32,
    checker_mode: f32,
    rot_turns: f32,
    flip_h: f32,
    flip_v: f32,
    _pad: f32,
};

@group(0) @binding(0) var img_tex: texture_2d<f32>;
@group(0) @binding(1) var img_samp: sampler;
@group(1) @binding(0) var<uniform> params: Params;

@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOut {
    var out: VertexOut;
    var p = pos;
    if params.flip_h > 0.5 {
        p.x = -p.x;
    }
    if params.flip_v > 0.5 {
        p.y = -p.y;
    }
    let turns = i32(round(params.rot_turns)) % 4;
    if turns == 1 {
        p = vec2<f32>(p.y, -p.x);
    } else if turns == 2 {
        p = vec2<f32>(-p.x, -p.y);
    } else if turns == 3 {
        p = vec2<f32>(-p.y, p.x);
    }
    out.pos = vec4<f32>(p * params.scale + params.offset, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var sample_col = textureSample(img_tex, img_samp, in.uv);
    if params.blur_sigma >= 0.05 || params.sharpen_sigma >= 0.05 {
        let texel = 1.0 / max(params.tex_size, vec2<f32>(1.0, 1.0));
        let r = clamp(max(params.blur_sigma, params.sharpen_sigma), 0.05, 3.0);
        var acc = sample_col * 0.25;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(texel.x * r, 0.0)) * 0.125;
        acc += textureSample(img_tex, img_samp, in.uv - vec2<f32>(texel.x * r, 0.0)) * 0.125;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(0.0, texel.y * r)) * 0.125;
        acc += textureSample(img_tex, img_samp, in.uv - vec2<f32>(0.0, texel.y * r)) * 0.125;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(texel.x * r, texel.y * r)) * 0.0625;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(-texel.x * r, texel.y * r)) * 0.0625;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(texel.x * r, -texel.y * r)) * 0.0625;
        acc += textureSample(img_tex, img_samp, in.uv + vec2<f32>(-texel.x * r, -texel.y * r)) * 0.0625;
        if params.sharpen_amount > 0.01 && params.sharpen_sigma >= 0.05 {
            let detail = sample_col.rgb - acc.rgb;
            sample_col = vec4<f32>(clamp(sample_col.rgb + detail * params.sharpen_amount, vec3<f32>(0.0), vec3<f32>(1.0)), sample_col.a);
        } else {
            sample_col = acc;
        }
    }
    if params.checker_mode > 0.5 && sample_col.a < 0.999 {
        let cell = vec2<i32>(floor(in.pos.xy / 12.0));
        let is_light = ((cell.x + cell.y) & 1) == 0;
        let bg = select(vec3<f32>(0.22, 0.22, 0.22), vec3<f32>(0.34, 0.34, 0.34), is_light);
        let blended = mix(bg, sample_col.rgb, sample_col.a);
        return vec4<f32>(blended, 1.0);
    }
    return sample_col;
}
"#;

impl GpuRenderer {
    pub fn new_for_window(window: Arc<Window>) -> Result<(Self, WindowGpuSurface), String> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|e| format!("surface: {e}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no compatible GPU adapter".to_string())?;

        // Request up to 16,384 px texture dimension instead of default 8,192 (E1)
        let max_texture_dim = adapter.limits().max_texture_dimension_2d.min(16_384);
        let mut limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        limits.max_texture_dimension_2d = max_texture_dim;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("iv-wgpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| format!("device: {e}"))?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        if size.width > 0 && size.height > 0 {
            surface.configure(&device, &config);
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iv-tex-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iv-params-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iv-image-shader"),
            source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER_WGSL.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iv-image-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout, &params_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iv-image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iv-bilinear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let params = ShaderParams {
            tex_size: [1.0, 1.0],
            viewport_size: [1280.0, 800.0],
            scale: [1.0, 1.0],
            offset: [0.0, 0.0],
            blur_sigma: 0.0,
            sharpen_sigma: 0.0,
            sharpen_amount: 0.0,
            checker_mode: 0.0,
            rot_turns: 0.0,
            flip_h: 0.0,
            flip_v: 0.0,
            _pad: 0.0,
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("iv-params-buf"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iv-params-bg"),
            layout: &params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);
        let win_surface = WindowGpuSurface {
            surface,
            config,
            needs_reconfigure: false,
            egui_renderer,
        };

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
                max_texture_dim,
                pipeline,
                bind_group_layout,
                sampler,
                params_buffer,
                params_bind_group,
            },
            win_surface,
        ))
    }

    pub fn create_child_surface(&self, window: Arc<Window>) -> Result<WindowGpuSurface, String> {
        let surface = self
            .instance
            .create_surface(Arc::clone(&window))
            .map_err(|e| format!("child surface: {e}"))?;
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        if size.width > 0 && size.height > 0 {
            surface.configure(&self.device, &config);
        }
        let egui_renderer = egui_wgpu::Renderer::new(&self.device, format, None, 1, false);
        Ok(WindowGpuSurface {
            surface,
            config,
            needs_reconfigure: false,
            egui_renderer,
        })
    }

    /// Uploads RGBA8 pixels into 1 or more GPU sub-texture tiles (`<= max_texture_dim`, E1).
    pub fn upload_tiled_rgba8(&self, rgba8: &[u8], width: u32, height: u32) -> Vec<GpuTile> {
        let tile_cap = self.max_texture_dim.max(1);
        let mut tiles = Vec::new();
        let mut y = 0u32;
        while y < height {
            let th = (height - y).min(tile_cap);
            let mut x = 0u32;
            while x < width {
                let tw = (width - x).min(tile_cap);
                let mut tile_bytes = vec![0u8; (tw as usize) * (th as usize) * 4];
                for row in 0..th as usize {
                    let src_off = ((y as usize + row) * (width as usize) + x as usize) * 4;
                    let dst_off = row * (tw as usize) * 4;
                    let len = (tw as usize) * 4;
                    if src_off + len <= rgba8.len() {
                        tile_bytes[dst_off..dst_off + len]
                            .copy_from_slice(&rgba8[src_off..src_off + len]);
                    }
                }
                let tex = self.device.create_texture_with_data(
                    &self.queue,
                    &wgpu::TextureDescriptor {
                        label: Some("iv-tile-tex"),
                        size: wgpu::Extent3d {
                            width: tw,
                            height: th,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                    wgpu::util::TextureDataOrder::LayerMajor,
                    &tile_bytes,
                );
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iv-tile-bg"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });

                let x0 = (x as f32 / width as f32) * 2.0 - 1.0;
                let x1 = ((x + tw) as f32 / width as f32) * 2.0 - 1.0;
                let y0 = 1.0 - (y as f32 / height as f32) * 2.0;
                let y1 = 1.0 - ((y + th) as f32 / height as f32) * 2.0;
                let verts = [
                    Vertex { pos: [x0, y0], uv: [0.0, 0.0] },
                    Vertex { pos: [x1, y0], uv: [1.0, 0.0] },
                    Vertex { pos: [x1, y1], uv: [1.0, 1.0] },
                    Vertex { pos: [x0, y1], uv: [0.0, 1.0] },
                ];
                let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("iv-tile-vb"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("iv-tile-ib"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                tiles.push(GpuTile {
                    bind_group,
                    vertex_buffer,
                    index_buffer,
                });
                x += tw;
            }
            y += th;
        }
        tiles
    }

    pub fn background_clear_color(bg: ViewerBackground) -> wgpu::Color {
        match bg {
            ViewerBackground::Black => wgpu::Color::BLACK,
            ViewerBackground::DarkGray => wgpu::Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 },
            ViewerBackground::White => wgpu::Color::WHITE,
            ViewerBackground::Checkerboard => wgpu::Color { r: 0.16, g: 0.16, b: 0.16, a: 1.0 },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_frame(
        &self,
        win_surface: &mut WindowGpuSurface,
        tiles: &[GpuTile],
        src_w: u32,
        src_h: u32,
        viewport: &ViewportState,
        bg: ViewerBackground,
        filters: FilterUniforms,
        egui_paint: Option<(&egui::Context, Vec<egui::ClippedPrimitive>, egui::TexturesDelta, f32)>,
    ) {
        if win_surface.config.width == 0 || win_surface.config.height == 0 {
            return;
        }
        if win_surface.needs_reconfigure {
            win_surface.surface.configure(&self.device, &win_surface.config);
            win_surface.needs_reconfigure = false;
        }

        let fit = ViewportLayoutCalculator::compute_fit_scale(
            src_w.max(1),
            src_h.max(1),
            viewport,
            viewport.display_mode,
        );
        let eff_scale = fit * viewport.zoom_scale;
        let (eff_w, eff_h) = if viewport.rotation_quarter_turns % 2 == 1 {
            (f64::from(src_h.max(1)), f64::from(src_w.max(1)))
        } else {
            (f64::from(src_w.max(1)), f64::from(src_h.max(1)))
        };
        let pw = f64::from(win_surface.config.width.max(1));
        let ph = f64::from(win_surface.config.height.max(1));
        let sx = ((eff_w * eff_scale) / pw) as f32;
        let sy = ((eff_h * eff_scale) / ph) as f32;
        let ox = ((viewport.pan_x * 2.0) / viewport.logical_width.max(1.0)) as f32;
        let oy = (-(viewport.pan_y * 2.0) / viewport.logical_height.max(1.0)) as f32;

        let params = ShaderParams {
            tex_size: [src_w.max(1) as f32, src_h.max(1) as f32],
            viewport_size: [pw as f32, ph as f32],
            scale: [sx, sy],
            offset: [ox, oy],
            blur_sigma: filters.blur_sigma,
            sharpen_sigma: filters.sharpen_sigma,
            sharpen_amount: filters.sharpen_amount,
            checker_mode: if bg == ViewerBackground::Checkerboard { 1.0 } else { 0.0 },
            rot_turns: f32::from(viewport.rotation_quarter_turns),
            flip_h: if viewport.flip_h { 1.0 } else { 0.0 },
            flip_v: if viewport.flip_v { 1.0 } else { 0.0 },
            _pad: 0.0,
        };
        self.queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let Ok(frame) = win_surface.surface.get_current_texture() else {
            win_surface.needs_reconfigure = true;
            return;
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("iv-frame-encoder"),
        });

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [win_surface.config.width, win_surface.config.height],
            pixels_per_point: egui_paint.as_ref().map(|p| p.3).unwrap_or(1.0),
        };

        let mut egui_prims = Vec::new();
        if let Some((_, prims, tdelta, _)) = egui_paint {
            for (id, delta) in &tdelta.set {
                win_surface.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
            }
            win_surface.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &prims,
                &screen_desc,
            );
            for id in &tdelta.free {
                win_surface.egui_renderer.free_texture(id);
            }
            egui_prims = prims;
        }

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iv-main-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Self::background_clear_color(bg)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut rpass = rpass.forget_lifetime();
            if !tiles.is_empty() {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(1, &self.params_bind_group, &[]);
                for tile in tiles {
                    rpass.set_bind_group(0, &tile.bind_group, &[]);
                    rpass.set_vertex_buffer(0, tile.vertex_buffer.slice(..));
                    rpass.set_index_buffer(tile.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.draw_indexed(0..6, 0, 0..1);
                }
            }
            if !egui_prims.is_empty() {
                win_surface.egui_renderer.render(&mut rpass, &egui_prims, &screen_desc);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
