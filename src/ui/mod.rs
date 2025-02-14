use wgpu::util::DeviceExt;

fl2rust_macro::include_ui!("src/ui/jadevid-ui-main.fl");

// USEFUL: https://github.com/fltk-rs/demos/tree/master/wgpu

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVert {
    position: [f32; 3],
    color: [f32; 3],
}

impl SimpleVert {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SimpleVert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const QUAD_VERTS: &[SimpleVert] = &[
    // TL
    SimpleVert {
        position: [-1.0, 1.0, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    // BL
    SimpleVert {
        position: [-1.0, -1.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    // BR
    SimpleVert {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    // TR
    SimpleVert {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
];

const QUAD_INDS: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct WgpuState<'a> {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    ind_count: u32,
}

impl WgpuState<'_> {
    pub async fn new(win: fltk::window::Window) -> Self {
        let (width, height) = (win.pixel_w() as _, win.pixel_h() as _);
        // Instance, surface, adapter, device
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(win).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let swapchain_format = surface.get_capabilities(&adapter).formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![swapchain_format],
        };
        surface.configure(&device, &surface_config);
        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[SimpleVert::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(swapchain_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDS),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            device,
            surface,
            surface_config,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            ind_count: QUAD_INDS.len() as u32,
        }
    }

    #[allow(unused)]
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        if self.valid_size() {
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn valid_size(&self) -> bool {
        self.surface_config.width > 0 && self.surface_config.height > 0
    }

    pub fn redraw(&self) {
        if !self.valid_size() {
            return;
        }

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // render()
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.ind_count, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
