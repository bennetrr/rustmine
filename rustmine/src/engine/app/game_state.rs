use crate::rustmine::entities::player::Player;
use crate::rustmine::generation::chunk::{Chunk, generate_mesh};
use crate::rustmine::generation::types::PlayerPos;
use crate::rustmine::generation::types::{BlockPos, ChunkPos};
use crate::rustmine::generation::world::World;
use crate::rustmine::ui::ui_chat::ChatUI;
use crate::engine::app::state::{EmptyResult, State, StateResult, TState};
use crate::engine::input::controller::{CameraController, ControllerAction};
use crate::engine::rendering::camera::FogUniform;
use crate::engine::rendering::crosshair::Crosshair;
use crate::engine::rendering::instance::{ChunkBlockGroup, InstanceRaw};
use crate::engine::rendering::lighting::LightUniform;
use crate::engine::rendering::model::{DrawModel, Vertex};
use crate::engine::rendering::{camera, model, resources, texture};
use crate::engine::ui::ui_factory::{UiComponents, load_fonts};
use cgmath::{InnerSpace, Vector2, Vector3};
use egui::{Align2, Color32, vec2};
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use std::time::SystemTime;
use wgpu::util::DeviceExt;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window};

const RENDER_DISTANCE_RADIUS: i32 = 5;

pub struct GameState {
    surface: Arc<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    chunk_groups: HashMap<Vector2<i32>, Vec<ChunkBlockGroup>>,
    world: World,
    world_name: String,
    is_paused: bool,
    is_chat_opened: bool,
    models: HashMap<String, Arc<model::Model>>,

    // GUI
    egui_context: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    egui_winit: egui_winit::State,
    chat_ui: ChatUI,

    state: Option<State>,

    // Entities
    pub player: Player,

    // Camera
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    crosshair: Crosshair,

    // Fog
    fog_bind_group: wgpu::BindGroup,
    fog_buffer: wgpu::Buffer,

    // Lighting
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,

    // Foliage
    foliage_render_pipeline: wgpu::RenderPipeline,

    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,
}

impl GameState {
    /// Initializes the entire game rendering state: WGPU surface, device, queue, pipelines,
    /// camera, lighting, world generation, and per-chunk instance buffers.
    ///
    /// # Errors
    /// Returns an error if the WGPU adapter/device request fails, a model fails to load,
    /// or the surface cannot be created.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        window: Arc<Window>,
        surface: Arc<wgpu::Surface<'static>>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        world_name: String,
    ) -> anyhow::Result<Self> {
        window.set_cursor_grab(if cfg!(target_os = "macos") {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::Confined
        })?;
        window.set_cursor_visible(false);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Texture Binding
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
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

        // World + block groups
        let world = World::load_world(&world_name)
            .map(|chunks| World::from_chunks(chunks, &world_name))
            .unwrap_or_else(|| World::new(&world_name));

        log::debug!("Load models");
        let mut chunk_groups: HashMap<Vector2<i32>, Vec<ChunkBlockGroup>> = HashMap::new();

        let mut models = HashMap::new();

        // Models insertion
        models.insert(
            "grass_model".to_string(),
            Arc::new(
                resources::load_model("grass.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        models.insert(
            "dirt_model".to_string(),
            Arc::new(
                resources::load_model("dirt.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        models.insert(
            "cobblestone_model".to_string(),
            Arc::new(
                resources::load_model(
                    "cobblestone.obj",
                    &device,
                    &queue,
                    &texture_bind_group_layout,
                )
                .await?,
            ),
        );

        models.insert(
            "stone_model".to_string(),
            Arc::new(
                resources::load_model("stone.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        // Wood
        models.insert(
            "oak_model".to_string(),
            Arc::new(
                resources::load_model("oak.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        models.insert(
            "spruce_model".to_string(),
            Arc::new(
                resources::load_model("spruce.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        models.insert(
            "birch_model".to_string(),
            Arc::new(
                resources::load_model("birch.obj", &device, &queue, &texture_bind_group_layout)
                    .await?,
            ),
        );

        // Leaves
        models.insert(
            "leaves_oak_model".to_string(),
            Arc::new(
                resources::load_model(
                    "leaves_oak.obj",
                    &device,
                    &queue,
                    &texture_bind_group_layout,
                )
                .await?,
            ),
        );

        models.insert(
            "leaves_birch_model".to_string(),
            Arc::new(
                resources::load_model(
                    "leaves_birch.obj",
                    &device,
                    &queue,
                    &texture_bind_group_layout,
                )
                .await?,
            ),
        );

        models.insert(
            "leaves_spruce_model".to_string(),
            Arc::new(
                resources::load_model(
                    "leaves_spruce.obj",
                    &device,
                    &queue,
                    &texture_bind_group_layout,
                )
                .await?,
            ),
        );

        // Other blocks
        models.insert(
            "tall_grass_model".to_string(),
            Arc::new(
                resources::load_model(
                    "tall_grass.obj",
                    &device,
                    &queue,
                    &texture_bind_group_layout,
                )
                .await?,
            ),
        );

        // Depth Buffer
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Lighting Area
        let light_uniform = LightUniform {
            position: [0.0, 10000.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // Camera
        let (eye, target, up, is_creative) = world.load_camera(&world_name).unwrap_or_else(|| {
            (
                [0.0, world.get_y(Vector2::new(0, -10)) as f32 + 4.0, -10.0],
                [0.0, 0.0, 0.0],
                Vector3::unit_y().into(),
                false,
            )
        });

        let camera = camera::Camera {
            eye: eye.into(),
            target: target.into(),
            up: up.into(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 80.0,
            znear: 0.1,
            zfar: 1000.0,
        };
        let camera_controller = CameraController::new(is_creative);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let player = Player::new(
            PlayerPos::new(camera.eye.x, camera.eye.y, camera.eye.z),
            camera,
            camera_controller,
        );

        // Crosshair
        let crosshair = Crosshair::new(
            &device,
            surface_format,
            config.width as f32,
            config.height as f32,
        );

        // Load World
        let player_chunk_pos = world
            .find_chunk_pos_for_block_pos(Vector3::new(player.pos.x as i32, 0, player.pos.z as i32))
            .unwrap();

        let visible_chunks: HashMap<ChunkPos, &Chunk> = world
            .chunks
            .iter()
            .filter(|(pos, _)| {
                (pos.x - player_chunk_pos.x).abs() <= RENDER_DISTANCE_RADIUS
                    && (pos.y - player_chunk_pos.y).abs() <= RENDER_DISTANCE_RADIUS
            })
            .map(|(pos, chunk)| (*pos, chunk))
            .collect();

        for (chunk_pos, chunk) in &visible_chunks {
            let mesh = generate_mesh(&chunk.blocks, *chunk_pos, [None, None, None, None]);

            // Chunk Groups Insertion
            chunk_groups.insert(
                *chunk_pos,
                vec![
                    ChunkBlockGroup::new(
                        models.get("grass_model").unwrap().clone(),
                        mesh.grass,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Grass Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("dirt_model").unwrap().clone(),
                        mesh.dirt,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Dirt Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("cobblestone_model").unwrap().clone(),
                        mesh.cobblestone,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Cobblestone Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("stone_model").unwrap().clone(),
                        mesh.stone,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Stone Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    // Wood
                    ChunkBlockGroup::new(
                        models.get("oak_model").unwrap().clone(),
                        mesh.oak,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Oak Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("spruce_model").unwrap().clone(),
                        mesh.spruce,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Spruce Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("birch_model").unwrap().clone(),
                        mesh.birch,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Birch Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    // Leaves
                    ChunkBlockGroup::new(
                        models.get("leaves_oak_model").unwrap().clone(),
                        mesh.leaves_oak,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Leaves Oak Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("leaves_birch_model").unwrap().clone(),
                        mesh.leaves_birch,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Leaves Birch Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    ChunkBlockGroup::new(
                        models.get("leaves_spruce_model").unwrap().clone(),
                        mesh.leaves_spruce,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Leaves Spruce Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                    // Other Blocks
                    ChunkBlockGroup::new(
                        models.get("tall_grass_model").unwrap().clone(),
                        mesh.tall_grass,
                        &device,
                        &format!(
                            "Chunk {:?}{:?} Tall Grass Instance Buffer",
                            chunk.position.x, chunk.position.y
                        ),
                    ),
                ],
            );
        }

        // Fog uniform
        let fog_uniform = FogUniform {
            color: [23.0 / 255.0, 62.0 / 255.0, 168.0 / 255.0], // must match render pass clear color
            start: 0.5 * world.generation_data.chunk_size as f32,
            end: RENDER_DISTANCE_RADIUS as f32 * world.generation_data.chunk_size as f32,
            _padding: [0.0; 3],
        };

        let fog_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fog Buffer"),
            contents: bytemuck::cast_slice(&[fog_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fog_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fog_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let fog_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog_bind_group"),
            layout: &fog_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fog_buffer.as_entire_binding(),
            }],
        });

        // Render Pipeline (!)
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                    Some(&fog_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/main.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
                Some(wgpu::Face::Back),
            )
        };

        // Lighting Pipeline
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                ],
                immediate_size: 0,
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
                Some(wgpu::Face::Back),
            )
        };

        // Foliage Pipeline
        let foliage_render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Foliage Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/foliage.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
                None,
            )
        };

        // GUI
        let egui_context = egui::Context::default();
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );

        egui_context.set_fonts(load_fonts().unwrap());

        let egui_winit = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        let chat_ui = ChatUI::new("root".to_string(), world_name.clone());

        Ok(Self {
            surface,
            device,
            queue,
            config,

            chunk_groups,
            world,
            world_name,
            is_paused: false,
            is_chat_opened: false,
            models,

            egui_context,
            egui_renderer,
            egui_winit,
            chat_ui,

            state: None,

            player,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            crosshair,

            fog_bind_group,
            fog_buffer,

            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,

            foliage_render_pipeline,

            depth_texture,
            render_pipeline,
            window,
        })
    }

    /// Recomputes and reuploads the instance buffers for all block groups for all chunks marked as dirty
    fn rebuild_block_groups(&mut self) {
        let dirty: Vec<_> = self
            .world
            .chunks
            .iter()
            .filter(|(_, chunk)| chunk.is_dirty)
            .filter(|(chunk_pos, _)| self.chunk_groups.contains_key(chunk_pos))
            .map(|(pos, _)| *pos)
            .collect();

        dirty
            .into_iter()
            .for_each(|pos| self.rebuild_block_groups_for_chunk(pos));
    }

    /// Recomputes and reuploads the instance buffers for all block groups for the chunk at `chunk_pos`.
    fn rebuild_block_groups_for_chunk(&mut self, chunk_pos: Vector2<i32>) {
        log::debug!("Rebuilding block groups for chunks {chunk_pos:?}");

        let neighbours = [
            self.world
                .chunks
                .get(&ChunkPos::new(chunk_pos.x + 1, chunk_pos.y))
                .map(|c| &c.blocks),
            self.world
                .chunks
                .get(&ChunkPos::new(chunk_pos.x - 1, chunk_pos.y))
                .map(|c| &c.blocks),
            self.world
                .chunks
                .get(&ChunkPos::new(chunk_pos.x, chunk_pos.y + 1))
                .map(|c| &c.blocks),
            self.world
                .chunks
                .get(&ChunkPos::new(chunk_pos.x, chunk_pos.y - 1))
                .map(|c| &c.blocks),
        ];
        let mesh = generate_mesh(
            &self.world.chunks.get(&chunk_pos).unwrap().blocks,
            chunk_pos,
            neighbours,
        );

        // Add instances to the chunk_groups
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[0].instances = mesh.grass;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[1].instances = mesh.dirt;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[2].instances = mesh.cobblestone;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[3].instances = mesh.stone;

        // Wood
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[4].instances = mesh.oak;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[5].instances = mesh.spruce;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[6].instances = mesh.birch;

        // Leaves
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[7].instances = mesh.leaves_oak;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[8].instances = mesh.leaves_birch;
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[9].instances = mesh.leaves_spruce;

        // Other Blocks
        self.chunk_groups.get_mut(&chunk_pos).unwrap()[10].instances = mesh.tall_grass;

        for group in self.chunk_groups.get_mut(&chunk_pos).unwrap() {
            group.rewrite_buffer(&self.device, &self.queue);
        }

        if let Some(chunk) = self.world.chunks.get_mut(&chunk_pos) {
            chunk.mark_as_clean();
        }
    }
}

impl TState for GameState {
    /// Handles window resize by reconfiguring the surface, depth texture, camera aspect ratio,
    /// and crosshair layout. No-ops if either dimension is zero.
    fn handle_resize(&mut self, width: u32, height: u32) -> EmptyResult {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.player.camera.aspect = self.config.width as f32 / self.config.height as f32;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.crosshair
                .resize(&self.queue, width as f32, height as f32);
        }

        Ok(())
    }

    /// Exits the application on `Escape`, otherwise forwards the key event to the camera controller.
    fn handle_key_press(&mut self, key: KeyCode, pressed: bool) -> StateResult {
        if key == KeyCode::Escape && pressed && !self.is_chat_opened {
            self.is_paused = !self.is_paused;

            if self.is_paused {
                self.window.set_cursor_grab(CursorGrabMode::None)?;
                self.window.set_cursor_visible(true);
            } else {
                self.window.set_cursor_grab(if cfg!(target_os = "macos") {
                    CursorGrabMode::Locked
                } else {
                    CursorGrabMode::Confined
                })?;
                self.window.set_cursor_visible(false);
            }
            Ok(None)
        } else if key == KeyCode::KeyT && pressed && !self.is_paused {
            self.is_chat_opened = true;
            self.window.set_cursor_grab(CursorGrabMode::None)?;
            self.window.set_cursor_visible(true);

            Ok(None)
        } else if key == KeyCode::Escape && pressed && self.is_chat_opened {
            self.is_chat_opened = false;
            self.window.set_cursor_grab(if cfg!(target_os = "macos") {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::Confined
            })?;
            self.window.set_cursor_visible(false);
            Ok(None)
        } else {
            if self.is_paused || self.is_chat_opened {
                return Ok(None);
            }
            self.player.handle_key(key, pressed);
            Ok(None)
        }
    }

    fn handle_mouse_button_press(&mut self, button: MouseButton, pressed: bool) -> StateResult {
        if self.is_paused {
            return Ok(None);
        }

        self.player.handle_mouse_button(button, pressed);
        Ok(None)
    }

    fn handle_mouse_movement(&mut self, dx: f64, dy: f64) -> EmptyResult {
        if self.is_paused {
            return Ok(());
        }

        self.player.handle_mouse(dx, dy);
        Ok(())
    }

    #[allow(unused_variables)]
    fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> EmptyResult {
        if self.is_paused || self.is_chat_opened {
            let _ = self.egui_winit.on_window_event(&self.window, event);
        }

        Ok(())
    }

    /// Processes camera controller actions (block destroy/place), then uploads
    /// the updated camera and light uniforms to the GPU.
    fn update(&mut self) -> StateResult {
        if self.is_paused {
            self.player.camera_controller.reset_input();
            return if self.state.is_some() {
                Ok(self.state.take())
            } else {
                Ok(None)
            };
        }

        if let Some([x, y, z]) = self.chat_ui.pending_teleport.take() {
            self.player.teleport(x, y, z);
        }

        if self.is_chat_opened {
            self.player.camera_controller.reset_input();
        }

        if !self.is_chat_opened {
            let is_generated = self.player.camera_controller.is_generated;
            let action = self.player.update(&self.world, is_generated);

            match action {
                ControllerAction::DestroyBlock => {
                    self.player.destroy_targeted_block(&mut self.world)
                }
                ControllerAction::CreateBlock(block_type) => self
                    .player
                    .create_targeted_block(&mut self.world, block_type),
                ControllerAction::None => {}
            }
        }
        self.rebuild_block_groups();

        self.camera_uniform.update_view_proj(&self.player.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );

        // Fog
        let (u, s) = (
            [10.0 / 255.0, 10.0 / 255.0, 10.0 / 255.0],
            [240.0 / 255.0, 240.0 / 255.0, 240.0 / 255.0],
        );
        let t = (((self.player.camera.eye.y + 15.0) / (5.0)) + 0.5).clamp(0.0, 1.0);
        let fog_color = [
            u[0] + (s[0] - u[0]) * t,
            u[1] + (s[1] - u[1]) * t,
            u[2] + (s[2] - u[2]) * t,
        ];

        let fog_uniform = FogUniform {
            color: fog_color,
            start: 0.5 * self.world.generation_data.chunk_size as f32,
            end: (RENDER_DISTANCE_RADIUS - 1) as f32 * self.world.generation_data.chunk_size as f32,
            _padding: [0.0; 3],
        };

        self.queue
            .write_buffer(&self.fog_buffer, 0, bytemuck::cast_slice(&[fog_uniform]));

        Ok(None)
    }

    /// Renders a full frame: all visible chunk block groups with lighting, followed by the crosshair.
    /// Skips rendering if the surface is not yet configured.
    ///
    /// # Errors
    /// Returns an error if the WGPU device is lost.
    fn render(&mut self) -> EmptyResult {
        self.window.request_redraw();

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Lost device"),
        };

        // The compositor may resize the surface (e.g. maximizing on X11) before a
        // `Resized` event reaches us. In that case the acquired color texture is already
        // at the new size while `config`/`depth_texture` still hold the old one, which
        // makes the render pass fail validation (differing attachment sizes). Resync the
        // config-derived state to the actual texture before rendering. The surface itself
        // is not reconfigured here since `output` is already valid at its real size.
        let (tex_width, tex_height) = (output.texture.width(), output.texture.height());
        if tex_width != self.config.width || tex_height != self.config.height {
            self.config.width = tex_width;
            self.config.height = tex_height;
            self.player.camera.aspect = tex_width as f32 / tex_height as f32;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.crosshair
                .resize(&self.queue, tex_width as f32, tex_height as f32);
        }

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let player_chunk_pos = self.world.chunk_pos_for_block_pos(Vector3::new(
            self.player.camera.eye.x as i32,
            0,
            self.player.camera.eye.z as i32,
        ));
        // Generate Chunks procedurally
        self.world.request_chunks_in_runtime(
            Vector3::new(self.player.camera.eye.x, 0.0, self.player.camera.eye.z),
            RENDER_DISTANCE_RADIUS,
            &self.chunk_groups,
        );

        // Procedural ChunkBlockGroups Insertion
        for mesh in self.world.collect_generated_chunks() {
            self.chunk_groups.insert(
                mesh.pos,
                vec![
                    ChunkBlockGroup::new(
                        self.models.get("grass_model").unwrap().clone(),
                        mesh.grass,
                        &self.device,
                        &format!("Chunk {:?} Grass", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("dirt_model").unwrap().clone(),
                        mesh.dirt,
                        &self.device,
                        &format!("Chunk {:?} Dirt", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("cobblestone_model").unwrap().clone(),
                        mesh.cobblestone,
                        &self.device,
                        &format!("Chunk {:?} Cobblestone", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("stone_model").unwrap().clone(),
                        mesh.stone,
                        &self.device,
                        &format!("Chunk {:?} Stone", mesh.pos),
                    ),
                    // Wood
                    ChunkBlockGroup::new(
                        self.models.get("oak_model").unwrap().clone(),
                        mesh.oak,
                        &self.device,
                        &format!("Chunk {:?} Oak", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("spruce_model").unwrap().clone(),
                        mesh.spruce,
                        &self.device,
                        &format!("Chunk {:?} Spruce", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("birch_model").unwrap().clone(),
                        mesh.birch,
                        &self.device,
                        &format!("Chunk {:?} Birch", mesh.pos),
                    ),
                    // Leaves
                    ChunkBlockGroup::new(
                        self.models.get("leaves_oak_model").unwrap().clone(),
                        mesh.leaves_oak,
                        &self.device,
                        &format!("Chunk {:?} Leaves Oak", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("leaves_birch_model").unwrap().clone(),
                        mesh.leaves_birch,
                        &self.device,
                        &format!("Chunk {:?} Leaves Birch", mesh.pos),
                    ),
                    ChunkBlockGroup::new(
                        self.models.get("leaves_spruce_model").unwrap().clone(),
                        mesh.leaves_spruce,
                        &self.device,
                        &format!("Chunk {:?} Leaves Spruce", mesh.pos),
                    ),
                    // Other Blocks
                    ChunkBlockGroup::new(
                        self.models.get("tall_grass_model").unwrap().clone(),
                        mesh.tall_grass,
                        &self.device,
                        &format!("Chunk {:?} Tall Grass", mesh.pos),
                    ),
                ],
            );
        }
        {
            let t = ((-self.player.camera.eye.y - 10.0) / 10.0).clamp(0.0, 1.0) as f64;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 240.0 / 255.0 * (1.0 - t),
                            g: 240.0 / 255.0 * (1.0 - t),
                            b: 240.0 / 255.0 * (1.0 - t),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_bind_group(3, &self.fog_bind_group, &[]);

            let visible_chunks: Vec<_> = self
                .chunk_groups
                .iter()
                .filter(|(chunk_pos, _)| {
                    (chunk_pos.x - player_chunk_pos.x).abs() <= RENDER_DISTANCE_RADIUS
                        && (chunk_pos.y - player_chunk_pos.y).abs() <= RENDER_DISTANCE_RADIUS
                })
                .collect();

            // Pass 1 - opaque blocks
            render_pass.set_pipeline(&self.render_pipeline);
            for (_, groups) in &visible_chunks {
                for group in &groups[..groups.len() - 1] {
                    if group.instances.is_empty() {
                        continue;
                    }
                    render_pass.set_vertex_buffer(1, group.instance_buffer.slice(..));
                    render_pass.draw_model_instanced(
                        &group.model,
                        0..group.instances.len() as u32,
                        &self.camera_bind_group,
                        &self.light_bind_group,
                    );
                }
            }

            // Pass 2 - foliage
            render_pass.set_pipeline(&self.foliage_render_pipeline);
            for (_, groups) in &visible_chunks {
                let group = &groups[groups.len() - 1];
                if group.instances.is_empty() {
                    continue;
                }
                render_pass.set_vertex_buffer(1, group.instance_buffer.slice(..));
                render_pass.draw_model_instanced(
                    &group.model,
                    0..group.instances.len() as u32,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );
            }
            render_pass.set_pipeline(&self.light_render_pipeline);

            if !self.is_paused {
                self.crosshair.draw(&mut render_pass);
            }
        }

        // Pause menu
        if self.is_paused {
            let uic = UiComponents::new(
                self.config.width,
                self.config.height,
                self.egui_context.pixels_per_point(),
            );

            let mut raw_input = self.egui_winit.take_egui_input(&self.window);
            raw_input.focused = self.window.has_focus();

            let full_output = self.egui_context.run_ui(raw_input, |ctx| {
                let menu_width = uic.vw(65);
                let gap = uic.pt(8);
                let small_button_width = (menu_width - gap) / 2.0;

                egui::Area::new(egui::Id::new("pause_menu"))
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.set_width(menu_width);

                        if uic.button(ui, menu_width, "Back to Game").clicked() {
                            self.is_paused = false;
                            self.window
                                .set_cursor_grab(if cfg!(target_os = "macos") {
                                    CursorGrabMode::Locked
                                } else {
                                    CursorGrabMode::Confined
                                })
                                .unwrap();
                            self.window.set_cursor_visible(false);
                        }
                        ui.add_space(gap);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;

                            if uic.button(ui, small_button_width, "Achievements").clicked() {
                                log::info!("Clicked Achievements");
                            }

                            if uic.button(ui, small_button_width, "Statistics").clicked() {
                                log::info!("Clicked Statistics");
                            }
                        });

                        ui.add_space(gap);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;

                            if uic.button(ui, small_button_width, "Options").clicked() {
                                log::info!("Clicked Options");
                            }

                            if uic.button(ui, small_button_width, "Open to LAN").clicked() {
                                log::info!("Clicked Open to LAN");
                            }
                        });

                        ui.add_space(gap);

                        if uic
                            .button(ui, menu_width, "Save and Quit to Menu")
                            .clicked()
                        {
                            self.world
                                .save_world(&self.world.chunks, &self.world_name)
                                .expect("COULD NOT SAVE THE WORLD");
                            World::save_world_metadata(&self.world_name, SystemTime::now())
                                .expect("COULD NOT SAVE WORLD METADATA");
                            self.world
                                .save_camera(
                                    &self.player.camera,
                                    &self.world_name,
                                    self.player.camera_controller.is_creative,
                                )
                                .expect("COULD NOT SAVE CAMERA");
                            self.state = Some(State::MainMenu);
                        }
                    });
            });

            self.egui_winit
                .handle_platform_output(&self.window, full_output.platform_output);

            let paint_jobs = self
                .egui_context
                .tessellate(full_output.shapes, full_output.pixels_per_point);

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: full_output.pixels_per_point,
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            }

            self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            {
                let ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Pause UI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                self.egui_renderer.render(
                    &mut ui_render_pass.forget_lifetime(),
                    &paint_jobs,
                    &screen_descriptor,
                );
            }

            for id in &full_output.textures_delta.free {
                self.egui_renderer.free_texture(id);
            }
        }

        if !self.is_paused {
            let mut uic = UiComponents::new(
                self.config.width,
                self.config.height,
                self.egui_context.pixels_per_point(),
            );

            let mut raw_input = self.egui_winit.take_egui_input(&self.window);
            raw_input.focused = self.window.has_focus();

            let full_output = self.egui_context.run_ui(raw_input, |ctx| {
                let menu_width = uic.vw(50);

                let direction = (self.player.camera.target - self.player.camera.eye).normalize();
                let direction_y = direction.y * 90.0;
                // X+ will be north
                let direction_xz = direction.z.atan2(direction.x).to_degrees();
                // Chunk Pos
                let chunk_pos = self.world.chunk_pos_for_block_pos(BlockPos::new(
                    self.player.camera.eye.x as i32,
                    self.player.camera.eye.y as i32,
                    self.player.camera.eye.z as i32,
                ));
                egui::Area::new(egui::Id::new("coordinates_area"))
                    .anchor(Align2::LEFT_TOP, vec2(uic.pt(4), uic.pt(4)))
                    .show(ctx, |ui| {
                        ui.set_width(menu_width);
                        ui.label(
                            egui::RichText::new(format!(
                                "Position: X: {:.2}, Y: {:.2}, Z: {:.2}",
                                self.player.camera.eye.x,
                                self.player.camera.eye.y,
                                self.player.camera.eye.z
                            ))
                            .size(uic.pt(16))
                            .color(Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Direction: X: {:.2}, Y: {:.2}, Z: {:.2}, XZ {:.2}",
                                direction.x, direction_y, direction.z, direction_xz
                            ))
                            .size(uic.pt(16))
                            .color(Color32::WHITE),
                        );

                        ui.label(
                            egui::RichText::new(format!(
                                "Chunk Pos: X: {:?}, Z: {:?}",
                                chunk_pos.x, chunk_pos.y
                            ))
                            .size(uic.pt(16))
                            .color(Color32::WHITE),
                        );
                    });
                if !self.is_chat_opened {
                    self.chat_ui.render_chat_closed(ctx, &mut uic);
                } else {
                    self.chat_ui.render_chat_opened(ctx, &mut uic);
                }
            });

            self.egui_winit
                .handle_platform_output(&self.window, full_output.platform_output);

            if !self.is_paused && !self.is_chat_opened {
                self.window.set_cursor_visible(false);
            }

            let paint_jobs = self
                .egui_context
                .tessellate(full_output.shapes, full_output.pixels_per_point);

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: full_output.pixels_per_point,
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            }

            self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            {
                let ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Pause UI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                self.egui_renderer.render(
                    &mut ui_render_pass.forget_lifetime(),
                    &paint_jobs,
                    &screen_descriptor,
                );
            }

            for id in &full_output.textures_delta.free {
                self.egui_renderer.free_texture(id);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.player.camera_controller.is_generated = true;

        Ok(())
    }
}

/// Creates a [`wgpu::RenderPipeline`] with triangle list topology, back-face culling,
/// CCW front faces, and optional depth testing.
fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    cull_mode: Option<wgpu::Face>,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode,
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
