use ecolor::{Color32, Rgba};
use egui_wgpu::RenderState;
use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};
use log::info;
#[feature(duration_millis_float)]
use std::time::Duration;
use std::time::Instant;
use wgpu::{util::DeviceExt, Buffer};
use winit::{
    event::{MouseScrollDelta, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    camera::{self, Camera, CameraUniform},
    constants::{INDICES, VERTICES},
    enums::cell_assets::{self, import_assets, CellAssets},
    instance_data::{InstanceData, Palette},
    objects::Player,
    world::{World, WorldObject},
    Vertex,
};

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub render_pipeline: wgpu::RenderPipeline,
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
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
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
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    limits: wgpu::Limits::default(),

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
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
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
                entry_point: "vs_main",                           // 1.
                buffers: &[Vertex::desc(), InstanceData::desc()], // 2.
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

        let assets = import_assets().unwrap();

        let mut palette = Palette {
            values: [Rgba::RED; 16],
        };
        for i in (0..16) {
            if let Some(color) = assets.assets_color_vec.get(i) {
                palette.values[i] = color.clone();
            }
        }

        let mut world = World::init_world(assets);

        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Colors Buffer"),
            contents: bytemuck::cast_slice(&[palette]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

        let camera = Camera::create_camera_from_screen_size(
            size.width as f32,
            size.height as f32,
            0.1,
            100.0,
            0.0,
            Vec2::ZERO,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        );

        let instances = {
            let mut instances = vec![];
            for x in -10..10 {
                for y in -10..10 {
                    instances.push(InstanceData {
                        position: Vec2::new(x as f32, y as f32),
                        scale: 1.0,
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

        return Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
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
        };
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.update_matrix_from_screen_size(
                self.size.width as f32,
                self.size.height as f32,
                self.camera.near,
                self.camera.far,
            );
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
            "Running update_instance_buffer() took {}.",
            elapsed_time.as_secs_f32(),
        );
    }

    pub fn update_colors_buffer(&mut self) {}

    pub fn input(&mut self, event: &WindowEvent, delta_t: f32) -> bool {
        self.world.input(delta_t, event, self.mouse_position);

        if let WindowEvent::MouseWheel { delta, .. } = event {
            if let MouseScrollDelta::LineDelta(_, scrolled) = delta {
                info!("scrolled: {}", scrolled);

                if scrolled > &0.0 {
                    self.camera.zoom_factor -= 0.3;
                } else {
                    self.camera.zoom_factor += 0.3;
                }
                self.camera.update_matrix();
                self.camera.update_camera_buffer(&self.queue);
            }
        }

        if let WindowEvent::KeyboardInput { input, .. } = event {
            let mut direction = Vec2::ZERO;
            match input.virtual_keycode {
                Some(VirtualKeyCode::W) => {
                    direction.y = 1.0;
                }
                Some(VirtualKeyCode::A) => {
                    direction.x = -1.0;
                }
                Some(VirtualKeyCode::S) => {
                    direction.y = -1.0;
                }
                Some(VirtualKeyCode::D) => {
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

        if let WindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
        } = event
        {
            self.world.add_obj(Box::new(Player {
                name: "test player".to_string(),
                position: self.mouse_position,
            }))
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
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
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
            render_pass.set_bind_group(0, &self.camera.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instance_buffer_len as _);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
