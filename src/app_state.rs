use core::panic;
use ecolor::Rgba;
use glam::{Mat4, Vec2, Vec4};
use log::info;
use std::{fs, sync::Arc, time::Instant};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, ComputePipeline,
    ComputePipelineDescriptor, RenderPipeline, Texture, TextureDescriptor,
    TextureFormat, TextureUsages,
};
use winit::{
    dpi::PhysicalSize,
    event::{MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    camera::{Camera, CameraUniform},
    constants::{INDICES, VERTICES},
    enums::cell_assets::import_assets,
    instance_data::{InstanceData, Palette},
    world::World,
};
use crate::{constants::Vertex, enums::cell_assets::CellAssets};

pub struct State<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_bind_layout: wgpu::BindGroupLayout,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub world: World,
    pub camera: Camera,
    pub instances: Vec<InstanceData>,
    pub instance_buffer: Buffer,
    pub instance_buffer_len: usize,
    pub mouse_position: Vec2,
    pub colors_buffer: Buffer,
    pub camera_bind_group: BindGroup,
}

impl<'a> State<'a> {
    fn create_screen_sized_texture(size: PhysicalSize<u32>, device: &wgpu::Device) -> Texture {
        

        device.create_texture(&TextureDescriptor {
            label: Some("screen sized texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::Rgba8Unorm],
        })
    }

    fn init_compute_pipeline(
        device: &wgpu::Device,
        size: PhysicalSize<u32>,
    ) -> (ComputePipeline, BindGroup, BindGroupLayout) {
        let compute_shader_file = {
            match fs::read_to_string("assets/shaders/compute_render.wgsl") {
                Ok(str) => str,
                Err(_) => {
                    panic!("could't load shader at path: assets/shaders/compute_render.wgsl ")
                }
            }
        };
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sand compute shader"),
            source: wgpu::ShaderSource::Wgsl(compute_shader_file.into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None, // Not an array
                }],
                label: Some("out_texture_bind_group_layout"),
            });
        let output = State::create_screen_sized_texture(size, device);
        let view = output.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
            label: Some("out_texture_bind_group"),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute sand renderer layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline_desc = ComputePipelineDescriptor {
            label: Some("Compute sand render"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        };

        (
            device.create_compute_pipeline(&compute_pipeline_desc),
            compute_bind_group,
            compute_bind_group_layout,
        )
    }

    fn load_assets(device: &wgpu::Device) -> (CellAssets, Buffer) {
        let assets = import_assets().unwrap();
        let mut palette = Palette {
            values: [Rgba::RED; 16],
        };
        let assets_clone = assets.clone();
        for i in 0..16 {
            if let Some(color) = assets_clone.assets_color_vec.get(i) {
                palette.values[i] = *color;
            }
        }

        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Colors Buffer"),
            contents: bytemuck::cast_slice(&[palette]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        (assets.clone(), colors_buffer)
    }

    fn init_render_pipeline(
        device: &wgpu::Device,
        config: wgpu::SurfaceConfiguration,
        colors_buffer: &Buffer,
    ) -> (RenderPipeline, BindGroup, Buffer) {
        let shader_file = {
            match fs::read_to_string("assets/shaders/shader.wgsl") {
                Ok(str) => str,
                Err(_) => {
                    panic!("could't load shader at path: assets/shaders/shader.wgsl ")
                }
            }
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_file.into()),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });

        let camera_uniform = CameraUniform {
            view_proj: Mat4::ZERO.to_cols_array_2d(),
            position: Vec4::ZERO,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: colors_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_colors_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: Default::default(), // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        (render_pipeline, camera_bind_group, camera_buffer)
    }

    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        info!("{:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: wgpu::Limits::default(),

                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: Default::default(),
        };
        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_vertices = VERTICES.len() as u32;
        let num_indices = INDICES.len() as u32;

        let (assets, colors_buffer) = State::load_assets(&device);

        let (render_pipeline, camera_bind_group, camera_buffer) =
            State::init_render_pipeline(&device, config.clone(), &colors_buffer);

        let (compute_pipeline, compute_bind_group, compute_bind_layout) =
            State::init_compute_pipeline(&device, size);

        let world = World::init_world(assets.clone());

        let camera = Camera::create_camera_from_screen_size(
            size.width as f32,
            size.height as f32,
            0.1,
            100.0,
            0.0,
            Vec2::ZERO,
            camera_buffer,
        );

        let instances = {
            let mut instances = vec![];
            for x in -10..10 {
                for y in -10..10 {
                    instances.push(InstanceData {
                        position: Vec2::new(x as f32, y as f32),
                        color: 0,
                    })
                }
            }
            instances
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // Add COPY_DST
        });

        let instance_buffer_len = instances.len();

        Self {
            instance,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            compute_pipeline,
            compute_bind_group,
            compute_bind_layout,
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            world,
            camera,
            instances,
            instance_buffer,
            instance_buffer_len,
            mouse_position: Vec2::ZERO,
            colors_buffer,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .update_matrix_from_screen_size(self.size.width as f32, self.size.height as f32);
            self.camera.update_camera_buffer(&self.queue);
        }
    }

    pub fn update_instance_buffer(&mut self) {
        let mut game_objects = vec![];

        for obj in self.world.storage.iter() {
            let now = Instant::now();
            game_objects.append(&mut obj.render());
            let elapsed_time = now.elapsed();
            info!(
                "Running render() took {} seconds for {}.",
                elapsed_time.as_secs_f32(),
                obj.get_name(),
            );
        }

        let now = Instant::now();

        let instance_data_size = std::mem::size_of::<InstanceData>();

        let instances_num = self.instances.len() + game_objects.len();

        if instances_num > self.instance_buffer_len {
            let new_size = instance_data_size * instances_num;

            self.instance_buffer.destroy();
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: new_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // Add STORAGE if needed
                mapped_at_creation: false,
            });
            self.instance_buffer_len = instances_num;
        }

        game_objects.append(&mut self.instances);

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&game_objects),
        );
        let elapsed_time = now.elapsed();
        info!(
            "Running update_instance_buffer() took {}, cells rendered {}.",
            elapsed_time.as_secs_f32(),
            game_objects.len()
        );
    }

    pub fn update_colors_buffer(&mut self) {}

    pub fn input(&mut self, event: &WindowEvent, delta_t: f32) -> bool {
        self.world.input(delta_t, event, self.mouse_position);

        if let WindowEvent::MouseWheel { delta, .. } = event {
            if let MouseScrollDelta::LineDelta(_, scrolled) = delta {
                info!("scrolled: {}", scrolled);

                if scrolled > &0.0 {
                    self.camera.zoom_factor -= 0.7;
                } else {
                    self.camera.zoom_factor += 0.7;
                }
                self.camera.update_matrix();
                self.camera.update_camera_buffer(&self.queue);
            }
        }

        if let WindowEvent::KeyboardInput { event, .. } = event {
            let mut direction = Vec2::ZERO;
            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    direction.y = 1.0;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    direction.x = -1.0;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    direction.y = -1.0;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    direction.x = 1.0;
                }
                _ => {}
            }
            if direction != Vec2::ZERO {
                self.camera.position += direction.normalize() * self.camera.zoom_factor;
                self.camera.update_matrix();

                info!("{}", self.camera.position);
                self.camera.update_camera_buffer(&self.queue);
            }
        }

        if let WindowEvent::CursorMoved { position, .. } = event {
            let position_in_game = self
                .camera
                .mouse_to_world(Vec2::new(position.x as f32, position.y as f32), self.size);

            self.mouse_position = position_in_game;

            info!("mouse pos in game: {}", position_in_game)
        }
        false
    }

    pub fn update(&mut self, delta_t: f32) {
        for obj in self.world.storage.iter_mut() {
            obj.as_mut().update(delta_t)
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.camera.update_camera_buffer(&self.queue);
        self.update_instance_buffer();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instance_buffer_len as _);
        }
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute render"),
                ..Default::default()
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[])
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
