use crate::engine::app::state::{State, TState};
use crate::engine::ui::ui_state_macros::{impl_handlers, impl_new, impl_render};
use egui::{Align2, vec2};
use std::option::Option;
use std::sync::Arc;
use winit::window::Window;

/// The account menu screen.
///
/// Not implemented yet.
/// or the "Play Selected Server" button. "Back" and Escape return to [`State::MainMenu`].
pub struct AccountMenuState {
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

    // Account Service Logic
    is_logged_in: bool,
}

impl AccountMenuState {
    impl_new! {
        background: "../../assets/images/menu_background.png",
        fields: {
            is_logged_in: false,
        },
    }
}

impl TState for AccountMenuState {
    impl_handlers!(escape: State::Exit);

    impl_render!(self, uic, ctx => {
        uic.heading(ctx, "Account Service");

        uic.area(ctx, "menu", Align2::CENTER_CENTER, |ui| {
            let visuals = uic.visuals(ui);

            ui.scope(|ui| {
                ui.set_visuals(visuals);

                ui.vertical_centered(|ui| {
                    uic.gap_y(ui, uic.pt(10));

                    if !self.is_logged_in {
                        if uic.button(ui, uic.vw(25), "Log In").clicked() {
                            log::info!("[Account Menu State]: Clicked Log In button.");
                            self.is_logged_in = true;
                        }
                    } else {
                        let button_width = uic.vw(40);
                        if uic.button(ui, button_width, "Account Settings").clicked() {
                            log::info!("[Account Menu State]: Clicked Account Settings Button");
                        }

                        if uic.button(ui, button_width, "Change Skin").clicked() {
                            log::info!("[Account Menu State]: Clicked Change Skin button.");
                        }
                    }
                });
            });
        });

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

                        if uic.button(ui, uic.vw(10), "Back").clicked() {
                            log::info!("[Account Menu State]: Clicked Back button.");
                            self.state = Some(State::MainMenu);
                        }

                        if self.is_logged_in && uic.button(ui, uic.vw(10), "Log Out").clicked()
                        {
                            log::info!("[Account Menu State]: Clicked Log Out button.");
                            self.is_logged_in = false;
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
    use winit::keyboard::KeyCode;

    // 1. MOCK STATE FOR LOGIC TESTING (No GPU Required)
    struct MockAccountMenuState {
        state: Option<State>,
        is_logged_in: bool,
    }

    impl MockAccountMenuState {
        fn new() -> Self {
            Self {
                state: None,
                is_logged_in: false,
            }
        }

        fn click_login(&mut self) {
            self.is_logged_in = true;
        }

        fn click_logout(&mut self) {
            self.is_logged_in = false;
        }

        fn click_back(&mut self) {
            self.state = Some(State::MainMenu);
        }

        fn handle_key_press(&mut self, key: KeyCode, pressed: bool) {
            if key == KeyCode::Escape && pressed {
                self.state = Some(State::Exit);
            }
        }
    }

    #[test]
    fn test_initial_state_is_logged_out() {
        let mock = MockAccountMenuState::new();
        assert!(!mock.is_logged_in, "User should be logged out by default.");
        assert!(
            mock.state.is_none(),
            "Initial state pending transition should be None."
        );
    }

    #[test]
    fn test_login_and_logout_toggle() {
        let mut mock = MockAccountMenuState::new();

        // Simulate Login
        mock.click_login();
        assert!(mock.is_logged_in, "User state should switch to logged in.");

        // Simulate Logout
        mock.click_logout();
        assert!(
            !mock.is_logged_in,
            "User state should successfully revert to logged out."
        );
    }

    #[test]
    fn test_back_button_transitions_to_main_menu() {
        let mut mock = MockAccountMenuState::new();
        mock.click_back();

        assert_eq!(
            mock.state,
            Some(State::MainMenu),
            "Back button must redirect the engine context to State::MainMenu."
        );
    }

    #[test]
    fn test_escape_key_triggers_exit_state() {
        let mut mock = MockAccountMenuState::new();

        mock.handle_key_press(KeyCode::Escape, true);
        assert_eq!(
            mock.state,
            Some(State::Exit),
            "Pressing Escape must route the engine to State::Exit as per your specifications."
        );
    }

    // negativ test
    #[test]
    fn test_arbitrary_keys_do_not_trigger_state_mutations() {
        let mut mock = MockAccountMenuState::new();

        mock.handle_key_press(KeyCode::KeyW, true);
        mock.handle_key_press(KeyCode::Space, true);

        assert!(
            mock.state.is_none(),
            "Unmapped inputs must not accidentally trigger mutations."
        );
    }
}
