use crate::engine::app::state::{State, TState};
use crate::engine::ui::ui_state_macros::{impl_handlers, impl_new, impl_render};
use egui::{Align2, vec2};
use std::option::Option;
use std::sync::Arc;
use winit::window::Window;

/// The multiplayer world selection screen.
///
/// Not implemented yet.
/// Renders a scrollable list of servers over a tiled dirt background.
/// A server can be selected with a single click and launched with a double click
/// or the "Play Selected Server" button. "Back" and Escape return to [`State::MainMenu`].
pub struct MultiplayerState {
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
}

impl MultiplayerState {
    impl_new! {
        background: "../../assets/images/menu_background.png",
        fields: {},
    }
}

impl TState for MultiplayerState {
    impl_handlers!(escape: State::Exit);

    impl_render!(self, uic, ctx => {
        uic.heading(ctx, "Join a Multiplayer World");

        uic.area_offset(
            ctx,
            "btn_footer",
            Align2::CENTER_BOTTOM,
            vec2(0.0, -20.0),
            |ui| {
                let visuals = uic.visuals(ui);

                ui.scope(|ui| {
                    ui.set_visuals(visuals);
                    ui.horizontal_centered(|ui| {
                        uic.gap_x(ui, uic.pt(10));

                        if uic.button(ui, uic.pt(135), "Back").clicked() {
                            log::info!("[Multiplayer Menu State]: Clicked Cancel button.");
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
    fn test_handle_key_press_escape_exits() {
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

            let mut multiplayer_state =
                MultiplayerState::new(window, surface, device, queue, config)
                    .await
                    .expect("Failed to initialize MultiplayerState instance context");

            let key_action = multiplayer_state.handle_key_press(KeyCode::Escape, true);
            assert!(
                key_action.is_ok(),
                "Expected safe key action mapping evaluation"
            );

            let wrapped_state = key_action.unwrap();
            assert!(
                wrapped_state.is_some(),
                "Expected explicit state transition variant wrapper on Escape key"
            );
            assert_eq!(
                wrapped_state.unwrap(),
                State::Exit,
                "Escape key must immediately transition into State::Exit"
            );
        });
    }

    #[test]
    fn test_handle_key_press_unmapped_ignores() {
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

            let mut multiplayer_state =
                MultiplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            let ignore_press = multiplayer_state.handle_key_press(KeyCode::KeyW, true);
            assert!(
                ignore_press.unwrap().is_none(),
                "Arbitrary keys must not trigger structural state changes"
            );

            let ignore_release = multiplayer_state.handle_key_press(KeyCode::Escape, false);
            assert!(
                ignore_release.unwrap().is_none(),
                "Releasing Escape key must not mutate routing scopes"
            );
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

            let mut multiplayer_state =
                MultiplayerState::new(window, surface, device, queue, config)
                    .await
                    .unwrap();

            let resize_result = multiplayer_state.handle_resize(1920, 1080);
            assert!(
                resize_result.is_ok(),
                "Expected sizing mutation context execution to return EmptyResult Ok"
            );
            assert_eq!(
                multiplayer_state.config.width, 1920,
                "Internal width parameter update mismatch"
            );
            assert_eq!(
                multiplayer_state.config.height, 1080,
                "Internal height parameter update mismatch"
            );

            let original_width = multiplayer_state.config.width;
            let zero_resize = multiplayer_state.handle_resize(0, 500);
            assert!(zero_resize.is_ok());
            assert_eq!(
                multiplayer_state.config.width, original_width,
                "Sizing configuration context should ignore empty bounds components"
            );
        });
    }
}
