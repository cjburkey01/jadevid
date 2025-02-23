use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

fl2rust_macro::include_ui!("src/ui/jadevid-ui-main.fl");

// USEFUL: https://github.com/fltk-rs/demos/tree/master/wgpu

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVert {
    position: Vec3,
    tex_coords: Vec2,
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
                    offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const QUAD_VERTS: &[SimpleVert] = &[
    // TL
    SimpleVert {
        position: Vec3::new(-1.0, 1.0, 0.0),
        tex_coords: Vec2::ZERO,
    },
    // BL
    SimpleVert {
        position: Vec3::new(-1.0, -1.0, 0.0),
        tex_coords: Vec2::Y,
    },
    // BR
    SimpleVert {
        position: Vec3::new(1.0, -1.0, 0.0),
        tex_coords: Vec2::ONE,
    },
    // TR
    SimpleVert {
        position: Vec3::new(1.0, 1.0, 0.0),
        tex_coords: Vec2::X,
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
    diffuse_bind_group: wgpu::BindGroup,
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

        // Texture
        let (diffuse_bind_group, texture_bind_group_layout) =
            Self::make_texture(&device, &queue, width, height);

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
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

        // Mesh data
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
            diffuse_bind_group,
        }
    }

    #[allow(unused)]
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        (self.diffuse_bind_group, _) = Self::make_texture(&self.device, &self.queue, width, height);
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
            rpass.set_bind_group(0, &self.diffuse_bind_group, &[]); // image
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.ind_count, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn make_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB, so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });
        let mut diffuse_rgba = vec![0x11 as u8; (width * height) as usize * 4];
        for yyy in 0..(height as usize / 4) {
            for xxx in 0..(width as usize / 4) {
                diffuse_rgba[16 * (xxx + yyy * width as usize) + 0] = 0x55;
                diffuse_rgba[16 * (xxx + yyy * width as usize) + 1] = 0x55;
                diffuse_rgba[16 * (xxx + yyy * width as usize) + 2] = 0x55;
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );
        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }),
            texture_bind_group_layout,
        )
    }
}
