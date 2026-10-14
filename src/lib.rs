// mod camera;
// mod instance;
// mod model;
// mod resources;
// mod texture;

// mod game;

// use anyhow::Result;
// use cgmath::prelude::*;
// use fs_extra::copy_items;
// use fs_extra::dir::CopyOptions;
// use std::{env, sync::Arc};
// use wgpu::util::DeviceExt;
// use winit::{
//     application::ApplicationHandler,
//     dpi::{LogicalSize, PhysicalSize},
//     event::WindowEvent,
//     event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
//     window::{Window, WindowId},
// };

// use crate::{
//     camera::{Camera, CameraUniform},
//     game::world::{Tile, World},
//     instance::{Instance, InstanceRaw},
//     model::{Model, Vertex as MVertex},
//     texture::Texture,
// };

// struct App {
//     window: Option<Arc<Window>>,
//     state: Option<State>,
// }

// struct State {
//     surface: wgpu::Surface<'static>,
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     config: wgpu::SurfaceConfiguration,
//     size: PhysicalSize<u32>,
//     render_pipeline: wgpu::RenderPipeline,
//     diffuse_bind_group: wgpu::BindGroup,
//     diffuse_texture: Texture,
//     depth_texture: Texture,
//     camera: Camera,
//     camera_uniform: CameraUniform,
//     camera_buffer: wgpu::Buffer,
//     camera_bind_group: wgpu::BindGroup,
//     instances: Vec<Instance>,
//     instance_buffer: wgpu::Buffer,
//     obj_model: Model,
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     tex_coords: [f32; 2],
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub enum CompareFunction {
//     Undefined = 0,
//     Never = 1,
//     Less = 2,
//     Equal = 3,
//     LessEqual = 4,
//     Greater = 5,
//     NotEqual = 6,
//     GreaterEqual = 7,
//     Always = 8,
// }

// impl Default for App {
//     fn default() -> Self {
//         Self {
//             window: None,
//             state: None,
//         }
//     }
// }

// impl ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         if self.window.is_some() {
//             return;
//         }

//         let attributes = Window::default_attributes()
//             .with_title("My Graphics Engine")
//             .with_inner_size(LogicalSize::new(800.0, 600.0));

//         let window = Arc::new(
//             event_loop
//                 .create_window(attributes)
//                 .expect("Failed to create window"),
//         );

//         let state = match pollster::block_on(State::new(Arc::clone(&window))) {
//             Ok(state) => state,
//             Err(error) => {
//                 log::error!("Graphics initialization failed: {error}");
//                 event_loop.exit();
//                 return;
//             }
//         };

//         self.window = Some(window);
//         self.state = Some(state);

//         if let Some(window) = &self.window {
//             window.request_redraw();
//         }
//     }

//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         window_id: WindowId,
//         event: WindowEvent,
//     ) {
//         let Some(window) = self.window.as_ref() else {
//             return;
//         };

//         if window.id() != window_id {
//             return;
//         }

//         match event {
//             WindowEvent::CloseRequested => {
//                 event_loop.exit();
//             }

//             WindowEvent::Resized(size) => {
//                 if let Some(state) = self.state.as_mut() {
//                     state.resize(size);
//                 }

//                 window.request_redraw();
//             }

//             WindowEvent::RedrawRequested => {
//                 let Some(state) = self.state.as_mut() else {
//                     return;
//                 };

//                 match state.render() {
//                     Ok(()) => {}

//                     Err(wgpu::CurrentSurfaceTexture::Lost)
//                     | Err(wgpu::CurrentSurfaceTexture::Outdated) => {
//                         state.reconfigure();
//                         window.request_redraw();
//                     }

//                     Err(wgpu::CurrentSurfaceTexture::Timeout) => {
//                         log::warn!("Surface timeout");
//                     }

//                     Err(error) => {
//                         log::error!("Surface error: {error:?}");
//                     }
//                 }
//             }

//             _ => {}
//         }
//     }

//     fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
//         if let Some(window) = &self.window {
//             window.request_redraw();
//         }
//     }
// }

// impl State {
//     async fn new(window: Arc<Window>) -> Result<Self, String> {
//         let size = window.inner_size();

//         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::all(),
//             backend_options: wgpu::BackendOptions::default(),
//             flags: wgpu::InstanceFlags::default(),
//             memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
//             display: None,
//         });

//         let surface = instance
//             .create_surface(Arc::clone(&window))
//             .map_err(|e| format!("Failed to create surface: {e}"))?;

//         let adapter = instance
//             .request_adapter(&wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::HighPerformance,
//                 compatible_surface: Some(&surface),
//                 force_fallback_adapter: false,
//                 apply_limit_buckets: false,
//             })
//             .await
//             .map_err(|e| format!("Failed to find GPU adapter: {e}"))?;

//         let (device, queue) = adapter
//             .request_device(&wgpu::DeviceDescriptor {
//                 label: Some("Main Device"),
//                 required_features: wgpu::Features::empty(),
//                 required_limits: wgpu::Limits::default(),
//                 memory_hints: wgpu::MemoryHints::default(),
//                 experimental_features: wgpu::ExperimentalFeatures::disabled(),
//                 trace: wgpu::Trace::default(),
//             })
//             .await
//             .map_err(|e| format!("Failed to create device: {e}"))?;

//         let capabilities = surface.get_capabilities(&adapter);

//         let format = capabilities
//             .formats
//             .iter()
//             .copied()
//             .find(|format| format.is_srgb())
//             .or_else(|| capabilities.formats.first().copied())
//             .ok_or_else(|| "No supported surface formats".to_string())?;

//         let alpha_mode = capabilities
//             .alpha_modes
//             .first()
//             .copied()
//             .ok_or_else(|| "No supported alpha modes".to_string())?;

//         let config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format,
//             width: size.width.max(1),
//             height: size.height.max(1),
//             present_mode: wgpu::PresentMode::Fifo,
//             alpha_mode,
//             view_formats: vec![],
//             desired_maximum_frame_latency: 2,
//             color_space: wgpu::SurfaceColorSpace::Auto,
//         };

//         let camera = Camera::new(
//             (0.0, 8.0, 20.0).into(),
//             (0.0, 0.0, 0.0).into(),
//             cgmath::Vector3::unit_y(),
//             config.width as f32 / config.height as f32,
//             45.0,
//             0.1,
//             100.0,
//         );

//         let mut camera_uniform = CameraUniform::new();
//         camera_uniform.update_view_proj(&camera);

//         let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Camera Buffer"),
//             contents: bytemuck::cast_slice(&[camera_uniform]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         });

//         let camera_bind_group_layout =
//             device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                 entries: &[wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 }],
//                 label: Some("camera_bind_group_layout"),
//             });

//         let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &camera_bind_group_layout,
//             entries: &[wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: camera_buffer.as_entire_binding(),
//             }],
//             label: Some("camera_bind_group"),
//         });

//         let diffuse_bytes = include_bytes!("img/happy-tree.png");
//         let diffuse_texture =
//             Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

//         let texture_bind_group_layout =
//             device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                 entries: &[
//                     wgpu::BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: wgpu::ShaderStages::FRAGMENT,
//                         ty: wgpu::BindingType::Texture {
//                             multisampled: false,
//                             view_dimension: wgpu::TextureViewDimension::D2,
//                             sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                         },
//                         count: None,
//                     },
//                     wgpu::BindGroupLayoutEntry {
//                         binding: 1,
//                         visibility: wgpu::ShaderStages::FRAGMENT,
//                         ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                         count: None,
//                     },
//                 ],
//                 label: Some("texture_bind_group_layout"),
//             });

//         let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &texture_bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
//                 },
//             ],
//             label: Some("diffuse_bind_group"),
//         });

//         let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: Some("Triangle Shader"),
//             source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
//         });

//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("Render Pipeline Layout"),
//             bind_group_layouts: &[
//                 Some(&texture_bind_group_layout),
//                 Some(&camera_bind_group_layout),
//             ],
//             immediate_size: 0,
//         });

//         let depth_texture =
//             texture::Texture::create_depth_texture(&device, &config, "depth_texture");

//         let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("Triangle Pipeline"),
//             layout: Some(&pipeline_layout),
            // vertex: wgpu::VertexState {
            //     module: &shader,
            //     entry_point: Some("vs_main"),
            //     buffers: &[Some(model::ModelVertex::desc()), Some(InstanceRaw::desc())],
            //     compilation_options: Default::default(),
            // },
//             fragment: Some(wgpu::FragmentState {
//                 module: &shader,
//                 entry_point: Some("fs_main"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: config.format,
//                     blend: Some(wgpu::BlendState::REPLACE),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState::default(),
//             depth_stencil: Some(wgpu::DepthStencilState {
//                 format: texture::Texture::DEPTH_FORMAT,
//                 depth_write_enabled: Some(true),
//                 depth_compare: Some(wgpu::CompareFunction::Less),
//                 stencil: wgpu::StencilState::default(),
//                 bias: wgpu::DepthBiasState::default(),
//             }),
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//         let world = World::generate(200, 200, 42);

//         let mut instances = Vec::new();

//         for y in 0..world.height {
//             for x in 0..world.width {
//                 let tile = world.tiles[(y * world.width + x) as usize];

//                 let position = cgmath::Vector3 {
//                     x: x as f32,
//                     y: 0.0,
//                     z: y as f32,
//                 };

//                 let rotation = cgmath::Quaternion::from_axis_angle(
//                     cgmath::Vector3::unit_y(),
//                     cgmath::Deg(0.0),
//                 );

//                 let color = match tile {
//                     Tile::Land => [0.5, 0.3, 0.1],  // marrón
//                     Tile::Water => [0.0, 0.3, 0.8], // azul
//                 };

//                 let instance = Instance::new(position, rotation, color);

//                 instances.push(instance);
//             }
//         }

//         let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
//         let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Instance Buffer"),
//             contents: bytemuck::cast_slice(&instance_data),
//             usage: wgpu::BufferUsages::VERTEX,
//         });

//         let obj_model =
//             resources::resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
//                 .await
//                 .map_err(|e| format!("Error cargando cube.obj: {e:#}"))?;

//         let state = Self {
//             surface,
//             device,
//             queue,
//             config,
//             size,
//             render_pipeline,
//             diffuse_bind_group,
//             diffuse_texture,
//             depth_texture,
//             camera,
//             camera_uniform,
//             camera_buffer,
//             camera_bind_group,
//             instance_buffer,
//             instances,
//             obj_model,
//         };

//         if size.width > 0 && size.height > 0 {
//             state.surface.configure(&state.device, &state.config);
//         }

//         Ok(state)
//     }

//     fn resize(&mut self, new_size: PhysicalSize<u32>) {
//         self.size = new_size;

//         if new_size.width == 0 || new_size.height == 0 {
//             return;
//         }

//         self.depth_texture =
//             texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
//         self.config.width = new_size.width;
//         self.config.height = new_size.height;

//         self.surface.configure(&self.device, &self.config);
//     }

//     fn reconfigure(&mut self) {
//         if self.size.width == 0 || self.size.height == 0 {
//             return;
//         }

//         self.config.width = self.size.width;
//         self.config.height = self.size.height;

//         self.surface.configure(&self.device, &self.config);
//     }

//     fn render(&mut self) -> Result<(), wgpu::CurrentSurfaceTexture> {
//         if self.size.width == 0 || self.size.height == 0 {
//             return Ok(());
//         }

//         let output = match self.surface.get_current_texture() {
//             wgpu::CurrentSurfaceTexture::Success(texture) => texture,
//             wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
//             error => return Err(error),
//         };

//         let view = output
//             .texture
//             .create_view(&wgpu::TextureViewDescriptor::default());

//         let mut encoder = self
//             .device
//             .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                 label: Some("Main Render Encoder"),
//             });

//         {
//             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Main Render Pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &view,
//                     resolve_target: None,
//                     depth_slice: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.05,
//                             g: 0.07,
//                             b: 0.12,
//                             a: 1.0,
//                         }),
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
//                     view: &self.depth_texture.view,
//                     depth_ops: Some(wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(1.0),
//                         store: wgpu::StoreOp::Store,
//                     }),
//                     stencil_ops: None,
//                 }),
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//                 multiview_mask: None,
//             });

//             render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
//             render_pass.set_pipeline(&self.render_pipeline);
//             render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
//             render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

//             use crate::model::DrawModel;
//             let mesh = &self.obj_model.meshes[0];
//             let material = &self.obj_model.materials[mesh.material];

//             render_pass.draw_mesh_instanced(
//                 mesh,
//                 material,
//                 0..self.instances.len() as u32,
//                 &self.camera_bind_group,
//             );
//         }

//         self.queue.submit(Some(encoder.finish()));
//         self.queue.present(output);

//         Ok(())
//     }
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     env_logger::init();

//     let event_loop = EventLoop::new().expect("Failed to create event loop");

//     event_loop.set_control_flow(ControlFlow::Poll);

//     let mut app = App::default();

//     event_loop.run_app(&mut app).expect("Event loop failed");

//     println!("cargo:rerun-if-changed=res/*");

//     let out_dir = env::var("OUT_DIR")?;
//     let mut copy_options = CopyOptions::new();
//     copy_options.overwrite = true;
//     let mut paths_to_copy = Vec::new();
//     paths_to_copy.push("res/");
//     copy_items(&paths_to_copy, out_dir, &copy_options)?;

//     Ok(())
// }