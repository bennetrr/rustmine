//! Shared boilerplate for [`TState`](crate::engine::app::state::TState) implementations that render UI.

/// Generates the constructor that sets up egui, uploads the background texture
/// and builds the fullscreen background pipeline.
///
/// # Parameters
///
/// - `background`: Path to the background texture relative from the current file.
/// - `fields`: Initial values for extra struct fields, in the form of `name: value`.
///
/// # Example
///
/// ```ignore
/// impl ExampleState {
///     impl_new! {
///         background: "../../assets/images/background.png",
///         fields: {
///             extra_field: true
///         },
///     }
/// }
/// ```
macro_rules! impl_new {
    (
        background: $background_path:literal,
        fields: { $($field:ident : $value:expr),* $(,)? } $(,)?
    ) => {
        pub async fn new(
            window: std::sync::Arc<winit::window::Window>,
            surface: std::sync::Arc<wgpu::Surface<'static>>,
            device: wgpu::Device,
            queue: wgpu::Queue,
            config: wgpu::SurfaceConfiguration,
        ) -> anyhow::Result<Self> {
            window.set_cursor_grab(winit::window::CursorGrabMode::None)?;
            window.set_cursor_visible(true);

            // GUI
            let egui_context = egui::Context::default();
            let egui_renderer = egui_wgpu::Renderer::new(
                &device,
                config.format,
                egui_wgpu::RendererOptions::default(),
            );

            // Background
            let img_data = include_bytes!($background_path);
            let img = image::load_from_memory(img_data)?.into_rgba8();
            let (img_w, img_h) = img.dimensions();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Background Texture"),
                size: wgpu::Extent3d {
                    width: img_w,
                    height: img_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * img_w),
                    rows_per_image: Some(img_h),
                },
                wgpu::Extent3d {
                    width: img_w,
                    height: img_h,
                    depth_or_array_layers: 1,
                },
            );

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bg_bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
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

            let bg_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BG Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            let vertices: &[f32] = &[
                -1.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0,
                0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            ];

            use wgpu::util::DeviceExt;
            let bg_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("BG Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            // Shader
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("BG Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui.wgsl").into()),
            });

            let pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("BG Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

            let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("BG Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 4 * 4, // 4 floats
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    })],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
                multiview_mask: None,
            });

            // Set egui font
            egui_context.set_fonts(crate::engine::ui::ui_factory::load_fonts().unwrap());

            // Mouse handling
            let egui_winit = egui_winit::State::new(
                egui_context.clone(),
                egui::ViewportId::ROOT,
                &window,
                None,
                None,
                None,
            );

            Ok(Self {
                surface,
                device,
                queue,
                config,
                window,

                egui_context,
                egui_renderer,

                bg_pipeline,
                bg_bind_group,
                bg_vertex_buffer,

                state: None,
                egui_winit,

                $($field: $value,)*
            })
        }
    };
}

/// Generates common handlers for [`TState`](crate::engine::app::state::TState):
///
/// - [`handle_resize`](crate::engine::app::state::TState::handle_resize)
/// - [`handle_key_press`](crate::engine::app::state::TState::handle_key_press)
/// - [`handle_mouse_button_press`](crate::engine::app::state::TState::handle_mouse_button_press)
/// - [`handle_mouse_movement`](crate::engine::app::state::TState::handle_mouse_movement)
/// - [`handle_window_event`](crate::engine::app::state::TState::handle_window_event)
/// - [`update`](crate::engine::app::state::TState::update)
///
/// `escape_state` is the [`State`](crate::engine::app::state::State)
/// that pressing the escape key transitions to.
macro_rules! impl_handlers {
    (escape: $escape_state:expr $(,)?) => {
        fn handle_resize(
            &mut self,
            width: u32,
            height: u32,
        ) -> crate::engine::app::state::EmptyResult {
            if width > 0 && height > 0 {
                self.config.width = width;
                self.config.height = height;
                self.surface.configure(&self.device, &self.config);
            }

            Ok(())
        }

        fn handle_key_press(
            &mut self,
            key: winit::keyboard::KeyCode,
            pressed: bool,
        ) -> crate::engine::app::state::StateResult {
            if key == winit::keyboard::KeyCode::Escape && pressed {
                return Ok(Some($escape_state));
            }
            Ok(None)
        }

        fn handle_mouse_button_press(
            &mut self,
            _button: winit::event::MouseButton,
            _pressed: bool,
        ) -> crate::engine::app::state::StateResult {
            Ok(None)
        }

        fn handle_mouse_movement(
            &mut self,
            _dx: f64,
            _dy: f64,
        ) -> crate::engine::app::state::EmptyResult {
            Ok(())
        }

        fn handle_window_event(
            &mut self,
            event: &winit::event::WindowEvent,
        ) -> crate::engine::app::state::EmptyResult {
            let _ = self.egui_winit.on_window_event(&self.window, event);
            Ok(())
        }

        fn update(&mut self) -> crate::engine::app::state::StateResult {
            if self.state.is_some() {
                Ok(self.state.take())
            } else {
                Ok(None)
            }
        }
    };
}

/// Generates the [`render`](crate::engine::app::state::TState::render) method,
/// wrapping the per-screen egui layout in the common pipeline.
///
/// Following names are available in the body:
///
/// - `$uic`: The constructed [`UiComponents`](crate::engine::ui::ui_factory::UiComponents)
/// - `$ctx`: The egui context
/// - `$self`: The state instance
///
/// # Example
///
/// ```ignore
/// impl TState for ExampleState {
///     render_state!(self, uic, ctx => {
///         uic.heading(ctx, "Example heading");
///         // ...
///     });
/// }
/// ```
macro_rules! impl_render {
    ($self:ident, $uic:ident, $ctx:ident => { $($body:tt)* }) => {
        fn render(&mut $self) -> crate::engine::app::state::EmptyResult {
            let $uic = crate::engine::ui::ui_factory::UiComponents::new(
                $self.config.width,
                $self.config.height,
                $self.egui_context.pixels_per_point(),
            );
            $self.window.request_redraw();

            let output = match $self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t)
                | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Validation => return Ok(()),
                wgpu::CurrentSurfaceTexture::Outdated => {
                    $self.surface.configure(&$self.device, &$self.config);
                    return Ok(());
                }
                wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Lost device"),
            };

            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut raw_input = $self.egui_winit.take_egui_input(&$self.window);
            raw_input.focused = $self.window.has_focus();

            let full_output = $self.egui_context.run_ui(raw_input, |$ctx| {
                $($body)*
            });

            // Mouse handling
            $self
                .egui_winit
                .handle_platform_output(&$self.window, full_output.platform_output);

            let paint_jobs = $self
                .egui_context
                .tessellate(full_output.shapes, full_output.pixels_per_point);
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [$self.config.width, $self.config.height],
                pixels_per_point: full_output.pixels_per_point,
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                $self
                    .egui_renderer
                    .update_texture(&$self.device, &$self.queue, *id, &image_delta[0]);
            }

            let mut update_encoder =
                $self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Egui Update Encoder"),
                    });
            $self.egui_renderer.update_buffers(
                &$self.device,
                &$self.queue,
                &mut update_encoder,
                &paint_jobs,
                &screen_descriptor,
            );
            $self.queue.submit(std::iter::once(update_encoder.finish()));

            let mut encoder =
                $self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            {
                let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                _render_pass.set_pipeline(&$self.bg_pipeline);
                _render_pass.set_bind_group(0, &$self.bg_bind_group, &[]);
                _render_pass.set_vertex_buffer(0, $self.bg_vertex_buffer.slice(..));
                _render_pass.draw(0..6, 0..1);

                $self.egui_renderer.render(
                    &mut _render_pass.forget_lifetime(),
                    &paint_jobs,
                    &screen_descriptor,
                );
            }

            for id in &full_output.textures_delta.free {
                $self.egui_renderer.free_texture(id);
            }

            $self.queue.submit(std::iter::once(encoder.finish()));
            $self.queue.present(output);

            Ok(())
        }
    };
}

pub(crate) use {impl_handlers, impl_new, impl_render};
