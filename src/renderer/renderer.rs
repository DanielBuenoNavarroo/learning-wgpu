use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{
    camera::{Camera, CameraUniform},
    instance::TileInstance,
    mesh::{Mesh, Vertex},
    pipeline::Pipeline,
};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    window: Arc<Window>,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    pipeline: Pipeline,

    pub mesh: Mesh,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("Failed to find GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Renderer Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }))
        .expect("Failed to create device");

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let alpha_mode = capabilities.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let pipeline = Pipeline::new(&device, &config);

        surface.configure(&device, &config);

        let camera = Camera::new(
            cgmath::Point3::new(120.0, 150.0, 120.0),
            cgmath::Point3::new(50.0, 0.0, 50.0),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            1000.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let vertices = [
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [0.3, 0.6, 0.2],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                color: [0.3, 0.6, 0.2],
            },
            Vertex {
                position: [1.0, 0.0, 1.0],
                color: [0.3, 0.6, 0.2],
            },
            Vertex {
                position: [0.0, 0.0, 1.0],
                color: [0.3, 0.6, 0.2],
            },
        ];

        let indices: &[u16] = &[0, 2, 1, 0, 3, 2];

        let mesh = Mesh::new(&device, &vertices, indices);

        let mut instances = Vec::new();

        for y in 0..100 {
            for x in 0..100 {
                instances.push(TileInstance {
                    position: [x as f32, 0.0, y as f32],
                    color: [0.3, 0.6, 0.2],
                });
            }
        }

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            window,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            pipeline,

            mesh,
            instance_buffer,
            instance_count: instances.len() as u32,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) => tex,
            wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            err => {
                eprintln!("Surface error: {:?}", err);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.07,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            self.pipeline.draw(&mut pass, &self.camera_bind_group);
            pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            pass.draw_indexed(0..self.mesh.index_count, 0, 0..self.instance_count);
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
    }
}
