use crate::rustmine::generation::world::{World, WorldMetaData};
use crate::rustmine::saves::Save;
use crate::engine::app::state::{State, TState};
use crate::engine::ui::ui_factory::{COLOR_HOVERED, COLOR_TEXT};
use crate::engine::ui::ui_state_macros::{impl_handlers, impl_new, impl_render};
use chrono::{DateTime, Local};
use egui::{Align2, Color32, Stroke};
use std::option::Option;
use std::sync::Arc;
use std::time::SystemTime;
use winit::window::Window;

/// The singleplayer world selection screen.
///
/// Renders a scrollable list of saved worlds over a tiled dirt background.
/// A world can be selected with a single click and launched with a double click
/// or the "Play Selected World" button. "Create New World" transitions to
/// [`State::CreateWorldMenu`], "Back" and Escape return to [`State::MainMenu`].
pub struct SingleplayerState {
    surface: Arc<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,

    // GUI
    egui_context: egui::Context,
    egui_renderer: egui_wgpu::Renderer,

    // Background
    bg_pipeline: wgpu::RenderPipeline,
    bg_bind_group: wgpu::BindGroup,
    bg_vertex_buffer: wgpu::Buffer,

    // State and mouse handling
    state: Option<State>,
    egui_winit: egui_winit::State,

    // World selection
    worlds: Vec<Save>,
    selected_world_id: Option<String>,
}

impl SingleplayerState {
    impl_new! {
        background: "../../assets/images/menu_background.png",
        fields: {
            worlds: Save::list(),
            selected_world_id: None,
        },
    }
}

impl TState for SingleplayerState {
    impl_handlers!(escape: State::MainMenu);

    impl_render!(self, uic, ctx => {
            let border_light = Color32::from_rgb(255, 255, 255);
            let border_dark = Color32::from_rgb(34, 34, 34);

            uic.heading(ctx, "Select World");

            egui::Area::new(egui::Id::new("world_list"))
                .anchor(Align2::CENTER_TOP, egui::vec2(0.0, 120.0))
                .default_height(uic.vh(100) - uic.pt(240))
                .show(ctx, |ui| {
                    let visuals = ui.visuals().clone();
                    ui.set_visuals(visuals);

                    egui::ScrollArea::vertical()
                        .max_height(uic.vh(100) - uic.pt(240))
                        .show(ui, |ui| {
                            uic.gap_y(ui, uic.pt(10));
                            let mut world_selected = false;

                            let mut worlds_with_meta: Vec<(&_, WorldMetaData)> = self
                            .worlds
                            .iter()
                            .map(|world| {
                                let meta = World::load_world_metadata(&world.name)
                                    .unwrap_or(WorldMetaData::new(SystemTime::UNIX_EPOCH));
                                (world, meta)
                            })
                            .collect();

                            worlds_with_meta.sort_by_key(|b| std::cmp::Reverse(b.1.last_played));

                            for (world, metadata) in worlds_with_meta.iter() {
                                let is_selected =
                                    self.selected_world_id == Some(world.name.to_string());

                                let border_color = if is_selected {
                                    border_light
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let fill_color = if is_selected {
                                    COLOR_HOVERED
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let button = egui::Frame::new()
                                    .stroke(Stroke::new(2.0, border_color))
                                    .fill(fill_color)
                                    .inner_margin(egui::Margin::symmetric(10, 10))
                                    .outer_margin(egui::Margin::symmetric(10, 0))
                                    .show(ui, |ui| {
                                        ui.set_width(uic.vw(100) - uic.pt(44));
                                        ui.label(
                                            egui::RichText::new(world.name.clone())
                                                .color(COLOR_TEXT)
                                                .strong()
                                                .size(uic.button_font_size),
                                        );

                                        let date = if metadata.last_played == SystemTime::UNIX_EPOCH {
                                            "00:00 00-00-0000".to_string()
                                        } else {
                                            let dt: DateTime<Local> = metadata.last_played.into();
                                            dt.format("%H:%M %d-%m-%Y").to_string()
                                        };
                                        ui.label(
                                            egui::RichText::new(format!("Last played: {}", date))
                                                .color(COLOR_TEXT)
                                                .weak()
                                                .size(uic.pt(30)),
                                        );
                                    })
                                    .response
                                    .interact(egui::Sense::click());

                                if button.double_clicked() {
                                    log::info!(
                                        "[Singleplayer Menu State]: Double Clicked {} button.",
                                        world.name
                                    );
                                    self.state = Some(State::Game(world.name.to_string()));
                                } else if button.clicked() {
                                    log::info!(
                                        "[Singleplayer Menu State]: Clicked World 1 button."
                                    );
                                    self.selected_world_id = Some(world.name.to_string());
                                    world_selected = true;
                                }

                                if is_selected {
                                    button.highlight();
                                }
                            }

                            if ctx.input(|i| i.pointer.primary_clicked()) && !world_selected && !ctx.is_pointer_over_egui() {
                                self.selected_world_id = None;
                            }
                        });
                });

            uic.area_offset(
                ctx,
                "btn_footer",
                Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -20.0),
                |ui| {
                    let mut visuals = ui.visuals().clone();
                    visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, border_dark);
                    ui.set_visuals(visuals);

                    ui.scope(|ui| {
                        ui.horizontal_centered(|ui| {
                            uic.gap_x(ui, uic.pt(10));

                            if uic.button(ui, uic.vw(20), "Play Selected World").clicked()
                                && let Some(name) = &self.selected_world_id
                            {
                                log::info!(
                                    "[Singleplayer Menu State]: Clicked Play Selected World button."
                                );
                                self.state = Some(State::Game(name.clone()));
                            }

                            if uic.button(ui, uic.vw(20), "Create New World").clicked() {
                                log::info!(
                                    "[Singleplayer Menu State]: Clicked Create New World button."
                                );
                                self.state = Some(State::CreateWorldMenu)
                            }

                            if uic.button(ui, uic.vw(20), "Delete World").clicked()
                                && let Some(name) = &self.selected_world_id
                            {
                                log::info!(
                                    "[Singleplayer Menu State]: Clicked Delete World button."
                                );
                                match Save::get_by_name(&name.clone()).delete() {
                                    Ok(()) => {
                                    log::info!("[Singleplayer Menu State] Deleted world '{}'", name);
                                    self.worlds.retain(|w| &w.name != name);
                                    self.selected_world_id = None;
                                },
                                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                        log::warn!("[Singleplayer Menu State] World '{}' save dir already missing", name);
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "[Singleplayer Menu State] Could not delete world '{}' saves directory: {}",
                                            name, e
                                        );
                                    }
                                    }
                            }

                            if uic.button(ui, uic.vw(20), "Back").clicked() {
                                log::info!("[Singleplayer Menu State]: Clicked Cancel button.");
                                self.state = Some(State::MainMenu);
                            }
                        });
                    });
                },
            );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::MouseButton;
    use winit::event_loop::EventLoop;
    use winit::keyboard::KeyCode;
    use winit::window::Window;

    // Helper function to initialize a headless wgpu adapter context safely for testing
    async fn create_mock_wgpu_context()
    -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .ok()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Test_Mock_Device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .ok()?;

        Some((instance, adapter, device, queue))
    }

    #[test]
    fn test_handle_key_press_escape_returns_to_main_menu() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let context = create_mock_wgpu_context().await;
            if context.is_none() {
                return;
            }
            let (_, _, device, queue) = context.unwrap();

            let event_loop = EventLoop::new().unwrap();
            let window_attributes = Window::default_attributes().with_visible(false);

            // Allow deprecated window creation since we only utilize it within a headless test environment context
            #[allow(deprecated)]
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let surface = Arc::new(
                wgpu::Instance::default()
                    .create_surface(window.clone())
                    .unwrap(),
            );

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let mut singleplayer_state =
                SingleplayerState::new(window, surface, device, queue, config)
                    .await
                    .expect("Failed to initialize SingleplayerState instance context");

            // Verify that the Escape key correctly triggers a transition to the Main Menu
            let key_action = singleplayer_state.handle_key_press(KeyCode::Escape, true);
            assert!(key_action.is_ok());

            let wrapped_state = key_action.unwrap();
            assert!(wrapped_state.is_some());
            assert_eq!(
                wrapped_state.unwrap(),
                State::MainMenu,
                "Escape key must immediately transition into State::MainMenu"
            );
        });
    }

    #[test]
    fn test_handle_key_press_unmapped_ignored() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let context = create_mock_wgpu_context().await;
            if context.is_none() {
                return;
            }
            let (_, _, device, queue) = context.unwrap();

            let event_loop = EventLoop::new().unwrap();
            let window_attributes = Window::default_attributes().with_visible(false);

            #[allow(deprecated)]
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let surface = Arc::new(
                wgpu::Instance::default()
                    .create_surface(window.clone())
                    .unwrap(),
            );

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let mut singleplayer_state =
                SingleplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            // Arbitrary keys must not trigger structural state changes
            let ignore_press = singleplayer_state.handle_key_press(KeyCode::KeyS, true);
            assert!(ignore_press.unwrap().is_none());

            // Releasing the Escape key must not mutate routing scopes
            let ignore_release = singleplayer_state.handle_key_press(KeyCode::Escape, false);
            assert!(ignore_release.unwrap().is_none());
        });
    }

    #[test]
    fn test_handle_resize_dimensions() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let context = create_mock_wgpu_context().await;
            if context.is_none() {
                return;
            }
            let (_, _, device, queue) = context.unwrap();

            let event_loop = EventLoop::new().unwrap();
            let window_attributes = Window::default_attributes().with_visible(false);

            #[allow(deprecated)]
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let surface = Arc::new(
                wgpu::Instance::default()
                    .create_surface(window.clone())
                    .unwrap(),
            );

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let mut singleplayer_state =
                SingleplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            let resize_result = singleplayer_state.handle_resize(1024, 768);
            assert!(
                resize_result.is_ok(),
                "Expected sizing mutation context execution to return EmptyResult Ok"
            );
            assert_eq!(
                singleplayer_state.config.width, 1024,
                "Internal width parameter update mismatch"
            );
            assert_eq!(
                singleplayer_state.config.height, 768,
                "Internal height parameter update mismatch"
            );

            // Sizing configuration context should ignore empty bounds components
            let original_width = singleplayer_state.config.width;
            let zero_resize = singleplayer_state.handle_resize(0, 768);
            assert!(zero_resize.is_ok());
            assert_eq!(
                singleplayer_state.config.width, original_width,
                "Width parameter should remain unchanged on invalid zero dimensions"
            );
        });
    }

    #[test]
    fn test_update_consumes_state() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let context = create_mock_wgpu_context().await;
            if context.is_none() {
                return;
            }
            let (_, _, device, queue) = context.unwrap();

            let event_loop = EventLoop::new().unwrap();
            let window_attributes = Window::default_attributes().with_visible(false);

            #[allow(deprecated)]
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let surface = Arc::new(
                wgpu::Instance::default()
                    .create_surface(window.clone())
                    .unwrap(),
            );

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let mut singleplayer_state =
                SingleplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            // Initial state cycle update loops should evaluate to an empty None state context
            let tick_initial = singleplayer_state.update();
            assert!(tick_initial.unwrap().is_none());

            // Simulate UI interaction targeting a specific game world transition route
            let test_world_name = "TestWorld".to_string();
            singleplayer_state.state = Some(State::Game(test_world_name.clone()));

            // The consumed inner transition state option must be cleared cleanly to prevent routing feedback loops
            let tick_transition = singleplayer_state.update();
            assert_eq!(
                tick_transition.unwrap().unwrap(),
                State::Game(test_world_name),
                "Expected update lifecycle loop to pass the requested routing target"
            );
            assert!(
                singleplayer_state.state.is_none(),
                "The internal transition buffer must be cleared post consumption"
            );
        });
    }

    #[test]
    fn test_world_selection_and_stubs() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let context = create_mock_wgpu_context().await;
            if context.is_none() {
                return;
            }
            let (_, _, device, queue) = context.unwrap();

            let event_loop = EventLoop::new().unwrap();
            let window_attributes = Window::default_attributes().with_visible(false);

            #[allow(deprecated)]
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let surface = Arc::new(
                wgpu::Instance::default()
                    .create_surface(window.clone())
                    .unwrap(),
            );

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let mut singleplayer_state =
                SingleplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            // Check that the selected world parameter defaults cleanly to None on initialization
            assert!(singleplayer_state.selected_world_id.is_none());

            // Simulate explicit mock manual data selection assignment binding
            singleplayer_state.selected_world_id = Some("MyCustomSave".to_string());
            assert_eq!(
                singleplayer_state.selected_world_id.as_deref(),
                Some("MyCustomSave")
            );

            // Verify that unimplemented input peripheral trait methods safely evaluate to default empty Ok contexts
            let mouse_click = singleplayer_state.handle_mouse_button_press(MouseButton::Left, true);
            assert!(mouse_click.unwrap().is_none());

            let mouse_move = singleplayer_state.handle_mouse_movement(5.0, -5.0);
            assert!(mouse_move.is_ok());
        });
    }
}
