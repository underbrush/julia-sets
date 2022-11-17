use wgpu::{BindGroupLayoutDescriptor, TextureUsages};
use winit::{ event::*, window::Window };
use std::io;

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pos: [f32; 2],
    wind: [f32; 2],

    c: [f32; 2],
    r: f32,

    colors: [f32; 4],
    set_color: bool,
    iterations: u32,

    keys: [bool; 20],

    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let mut limits = wgpu::Limits::default();
        limits.max_push_constant_size = 128;

        let size = window.inner_size();

        // Stupid gpu setup
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS |
                    wgpu::Features::CLEAR_TEXTURE,
                limits,
                label: None,
            },
            None,
        ).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let pos = [0.0; 2];
        let wind = [2.0, 2.0 * (size.height as f32) / (size.width as f32)];

        let c = [-0.0, 0.0];
        let c_mag = (c[0] * c[0] + c[1] * c[1] as f32).sqrt();
        let r = ((1.0 + (1.0 - 4.0 * c_mag).sqrt()) / 2.0) + 0.01;

        let colors = [3.3, -0.003, 0.2, 1.0];

        let keys = [false; 20];

        /* #region SHADERS */
        let julia_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("julia.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[],
                push_constant_ranges: &[
                    wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..48,
                    }],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &julia_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &julia_shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            multiview: None,
        });
        /* #endregion */

        Self {
            surface,
            device,
            queue,
            config,
            size,

            pos,
            wind,

            c,
            r,

            colors,
            set_color: false,
            iterations: 100,

            keys,

            render_pipeline,
        }
    }

    fn update_c(&mut self, new_c: [f32; 2]) {
        self.c = new_c;
        let c_mag = (self.c[0] * self.c[0] + self.c[1] * self.c[1] as f32).sqrt();
        self.r = (1.0 + (1.0 + 4.0 * c_mag).sqrt()) / 2.0;
        self.r = self.r * self.r;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.wind = [
                self.wind[0],
                self.wind[0] * (new_size.height as f32) / (new_size.width as f32)
            ];
        }
    }

    pub fn update(&mut self) {
        if self.keys[0] {
            self.update_c([self.c[0], self.c[1] - self.wind[0] * 0.001]);
        } if self.keys[1] {
            self.update_c([self.c[0] - self.wind[0] * 0.001, self.c[1]]);
        } if self.keys[2] {
            self.update_c([self.c[0], self.c[1] + self.wind[0] * 0.001]);
        } if self.keys[3] {
            self.update_c([self.c[0] + self.wind[0] * 0.001, self.c[1]]);
        } if self.keys[4] {
            self.pos = [self.pos[0], self.pos[1] - self.wind[0] * 0.01];
        } if self.keys[5] {
            self.pos = [self.pos[0] - self.wind[0] * 0.01, self.pos[1]];
        } if self.keys[6] {
            self.pos = [self.pos[0], self.pos[1] + self.wind[0] * 0.01];
        } if self.keys[7] {
            self.pos = [self.pos[0] + self.wind[0] * 0.01, self.pos[1]];
        } if self.keys[8] {
            self.wind = [self.wind[0] * 0.99, self.wind[1] * 0.99];
        } if self.keys[9] {
            self.wind = [self.wind[0] / 0.99, self.wind[1] / 0.99];
        } if self.keys[10] {
            self.colors[0] += 0.01;
        } if self.keys[11] {
            self.colors[0] -= 0.01;
        } if self.keys[12] {
            self.colors[1] += 0.0001;
        } if self.keys[13] {
            self.colors[1] -= 0.0001;
        }
        if !self.set_color {
            if self.keys[14] {
                self.colors[2] = 1.0f32.min(self.colors[2] + 0.01);
            } if self.keys[15] {
                self.colors[2] = 0.0f32.max(self.colors[2] - 0.01);
            } if self.keys[16] {
                self.colors[3] = 1.0f32.min(self.colors[3] + 0.01);
            } if self.keys[17] {
                self.colors[3] = 0.0f32.max(self.colors[3] - 0.01);
            }
        } else {
            if self.keys[14] {
                self.colors[2] = (-1.0f32).max(self.colors[2] - 0.01);
            } if self.keys[15] {
                self.colors[2] = 0.0f32.min(self.colors[2] + 0.01);
            } if self.keys[16] {
                self.colors[3] = (-1.0f32).max(self.colors[3] - 0.01);
            } if self.keys[17] {
                self.colors[3] = 0.0f32.min(self.colors[3] + 0.01);
            }
        } if self.keys[18] && self.iterations > 1 {
            self.iterations -= 1;
        } if self.keys[19] {
            self.iterations += 1;
        }
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.size)
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &Window) -> bool {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == window.id() => if !self.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => return true,
                    WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        self.resize(**new_inner_size);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        return false
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    /* #region STUPID KEYBINDINGS */
                    VirtualKeyCode::Up => {
                        self.keys[0] = is_pressed; true
                    }
                    VirtualKeyCode::Left => {
                        self.keys[1] = is_pressed; true
                    }
                    VirtualKeyCode::Down => {
                        self.keys[2] = is_pressed; true
                    }
                    VirtualKeyCode::Right => {
                        self.keys[3] = is_pressed; true
                    }
                    VirtualKeyCode::W => {
                        self.keys[4] = is_pressed; true
                    },
                    VirtualKeyCode::A => {
                        self.keys[5] = is_pressed; true
                    },
                    VirtualKeyCode::S => {
                        self.keys[6] = is_pressed; true
                    },
                    VirtualKeyCode::D => {
                        self.keys[7] = is_pressed; true
                    }
                    VirtualKeyCode::C => {
                        self.keys[8] = is_pressed; true
                    },
                    VirtualKeyCode::X => {
                        self.keys[9] = is_pressed; true
                    },
                    VirtualKeyCode::T => {
                        self.keys[10] = is_pressed; true
                    },
                    VirtualKeyCode::Y => {
                        self.keys[11] = is_pressed; true
                    },
                    VirtualKeyCode::G => {
                        self.keys[12] = is_pressed; true
                    },
                    VirtualKeyCode::H => {
                        self.keys[13] = is_pressed; true
                    },
                    VirtualKeyCode::O => {
                        self.keys[14] = is_pressed; true
                    },
                    VirtualKeyCode::I => {
                        self.keys[15] = is_pressed; true
                    },
                    VirtualKeyCode::L => {
                        self.keys[16] = is_pressed; true
                    },
                    VirtualKeyCode::K => {
                        self.keys[17] = is_pressed; true
                    },
                    VirtualKeyCode::Return => {
                        if is_pressed {
                            self.set_color = !self.set_color;
                            self.colors[2] = -self.colors[2];
                            self.colors[3] = -self.colors[3];
                        }
                        true
                    },
                    /* #endregion */
                    VirtualKeyCode::N => {
                        self.keys[18] = is_pressed; true
                    },
                    VirtualKeyCode::M => {
                        self.keys[19] = is_pressed; true
                    },
                    _ => false,
                }
            },
            _ => false
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
            }
        );
        
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }
                        ),
                        store: true,
                    }
                })
            ],
            depth_stencil_attachment: None,
        };

        {
            let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(&[
                    self.pos[0] - self.wind[0],
                    self.pos[1] - self.wind[1],
                    self.pos[0] + self.wind[0],
                    self.pos[1] + self.wind[1],
                    self.c[0], self.c[1],
                    self.r * self.r,
                    (self.iterations as f32),
                    self.colors[0], self.colors[1],
                    self.colors[2], self.colors[3],
                ])
            );
            rpass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
