use std::iter;
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
};

// --- SHADER WGSL INTEGRADO ---
// Esto define cómo se ve la nebulosa
const SHADER_SOURCE: &str = r#"
struct Uniforms {
    time: f32,
    mouse_x: f32,
    mouse_y: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> utils: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) & 1) * 4.0 - 1.0;
    let y = f32(i32(in_vertex_index) & 2) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = utils.time * 0.2; // Velocidad
    let uv = in.uv;
    
    // Efecto fluido simple
    let wave = sin(uv.x * 3.0 + t) * cos(uv.y * 3.0 + t * 0.5);
    let mouse_dist = distance(uv, vec2<f32>(utils.mouse_x, utils.mouse_y));
    
    // Colores Lumen (Azul oscuro a violeta)
    let color1 = vec3<f32>(0.05, 0.05, 0.1); // Fondo
    let color2 = vec3<f32>(0.2, 0.4, 0.9);   // Acento
    
    let mix_val = smoothstep(-1.0, 1.0, wave + (1.0 - mouse_dist) * 0.5);
    let final_color = mix(color1, color2, mix_val);
    
    return vec4<f32>(final_color, 1.0);
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    mouse_x: f32,
    mouse_y: f32,
    padding: f32,
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    start_time: instant::Instant,
}

impl State {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::downlevel_webgl2_defaults(),
            label: None,
        }, None).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let uniforms = Uniforms { time: 0.0, mouse_x: 0.0, mouse_y: 0.0, padding: 0.0 };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self { surface, device, queue, config, size, render_pipeline, uniform_buffer, bind_group, start_time: instant::Instant::now() }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self, mouse: (f32, f32)) {
        let time = self.start_time.elapsed().as_secs_f32();
        let uniforms = Uniforms { time, mouse_x: mouse.0, mouse_y: mouse.1, padding: 0.0 };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: true },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[wasm_bindgen]
pub async fn run() {
    use winit::platform::web::WindowBuilderExtWebSys;
    use wasm_bindgen::JsCast;

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let event_loop = EventLoop::new();
    
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();
    let canvas = doc.get_element_by_id("lumen-canvas")
        .expect("No se encontró el canvas 'lumen-canvas'")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("El elemento no es un canvas");

    let window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window).await;
    let mut mouse_pos = (0.0, 0.0);

    event_loop.run(move |event, _, _| {
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => state.resize(size),
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                mouse_pos = ((position.x as f32 / state.size.width as f32) * 2.0 - 1.0, (position.y as f32 / state.size.height as f32) * 2.0 - 1.0);
            },
            Event::RedrawRequested(_) => {
                state.update(mouse_pos);
                state.render().ok();
            },
            Event::MainEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}

// --- ESTRUCTURA QUILL (PLUMA) ---
// Esto soluciona el error de referencia. Genera la clase "Quill" en el JS/TS
// para que la web pueda importarla sin fallar, aunque esté vacía por ahora.
#[wasm_bindgen]
pub struct Quill;

#[wasm_bindgen]
impl Quill {
    #[wasm_bindgen(constructor)]
    pub fn new(_canvas_id: &str) -> Quill { Quill }
    pub fn set_tilt(&mut self, _x: f32, _y: f32) {}
    pub fn tick(&mut self, _dt: f32) {}
    pub fn render(&mut self, _time: f32, _w: f32, _h: f32) {}
}
