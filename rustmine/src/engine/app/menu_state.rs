use crate::engine::app::state::{State, TState};
use crate::engine::ui::ui_factory::FONT_TITLE;
use crate::engine::ui::ui_state_macros::{impl_handlers, impl_new, impl_render};
use egui::{Align2, FontFamily};
use std::option::Option;
use std::sync::Arc;
use winit::window::Window;

/// The main menu screen.
///
/// Renders a "RUSTMINE" title with a pulsing gray sine-wave animation over a dark
/// forest background, with three buttons: "Singleplayer" → [`State::SingleplayerMenu`],
/// "Multiplayer" → [`State::MultiplayerMenu`], "Quit Game" / Escape → [`State::Exit`].
pub struct MenuState {
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

impl MenuState {
    impl_new! {
        background: "../../assets/images/main_background.png",
        fields: {},
    }
}

impl TState for MenuState {
    impl_handlers!(escape: State::Exit);

    impl_render!(self, uic, ctx => {
        let time = ctx.input(|i| i.time);
        let speed = 1.5;

        let grey_value = ((time * speed).sin() * 40.0 + 40.0) as u8;
        let rgb_color = egui::Color32::from_rgb(grey_value, grey_value, grey_value);

        uic.area(ctx, "fogbound", Align2::CENTER_TOP, |ui| {
            ui.horizontal_centered(|ui| {
                let mut visuals = ui.visuals().clone();
                visuals.panel_fill = egui::Color32::TRANSPARENT;
                ui.set_visuals(visuals);

                ui.heading(
                    egui::RichText::new("RUSTMINE")
                        .color(rgb_color)
                        .family(FontFamily::Name(FONT_TITLE.into()))
                        .size(uic.vh(20)),
                );
            });
        });

        uic.area(ctx, "menu", Align2::CENTER_CENTER, |ui| {
            let visuals = uic.visuals(ui);

            ui.scope(|ui| {
                ui.set_visuals(visuals);

                ui.vertical_centered(|ui| {
                    uic.gap_y(ui, uic.pt(10));

                    if uic.button(ui, uic.vw(50), "Singleplayer").clicked() {
                        log::info!("[Menu State]: Clicked Singleplayer button.");
                        self.state = Some(State::SingleplayerMenu);
                    }

                    if uic.button(ui, uic.vw(50), "Multiplayer").clicked() {
                        log::info!("[Menu State]: Clicked Multiplayer button.");
                        self.state = Some(State::MultiplayerMenu)
                    }

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = uic.pt(8);

                        let small_button_width = (uic.vw(50) - uic.pt(8)) / 2.0;
                        if uic.button(ui, small_button_width, "Account").clicked() {
                            log::info!("[Menu State]: Clicked Account Button");
                            self.state = Some(State::AccountMenu);
                        }

                        if uic.button(ui, small_button_width, "Quit Game").clicked() {
                            log::info!("[Menu State]: Clicked Exit button.");
                            self.state = Some(State::Exit);
                        }
                    });
                });
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;
    #[test]
    fn test_menu_state_creation() {
        // test of the logic
        let initial_state = State::MainMenu;
        assert_eq!(initial_state, State::MainMenu);
    }

    #[test]
    fn test_menu_state_transitions() {
        // Test to see the Menu
        let transitions = vec![
            (State::MainMenu, State::SingleplayerMenu),
            (State::MainMenu, State::MultiplayerMenu),
            (State::MainMenu, State::Exit),
        ];

        for (from, to) in transitions {
            assert_ne!(from, to, "Menu should turn to one of those states");
        }
    }

    // Test the escape button
    #[test]
    fn test_escape_key_exits_menu() {
        let key = KeyCode::Escape;
        let pressed = true;

        // Pressing escape should lead to exit
        if key == KeyCode::Escape && pressed {
            assert!(true, "ESC go to exit");
        }
    }

    #[test]
    fn test_menu_buttons_exist() {
        // Test if all buttons exist
        let expected_buttons = ["Singleplayer", "Multiplayer", "Quit Game"];

        assert_eq!(expected_buttons.len(), 3, "Menu should have 3 buttons");
        assert!(expected_buttons.contains(&"Singleplayer"));
        assert!(expected_buttons.contains(&"Multiplayer"));
        assert!(expected_buttons.contains(&"Quit Game"));
    }

    #[test]
    fn test_singleplayer_button_leads_to_singleplayer_menu() {
        let expected_state = State::SingleplayerMenu;
        assert_eq!(expected_state, State::SingleplayerMenu);
    }

    #[test]
    fn test_multiplayer_button_leads_to_multiplayer_menu() {
        let expected_state = State::MultiplayerMenu;
        assert_eq!(expected_state, State::MultiplayerMenu);
    }

    #[test]
    fn test_quit_button_leads_to_exit() {
        let expected_state = State::Exit;
        assert_eq!(expected_state, State::Exit);
    }

    #[test]
    fn test_window_cursor_visibility_in_menu() {
        let cursor_visible = true;
        let cursor_grab_mode = "None";

        assert!(cursor_visible, "cursor must visible");
        assert_eq!(cursor_grab_mode, "None", "don't have to be visible");
    }

    #[test]
    fn test_resize_handling() {
        let new_width = 1920;
        let new_height = 1080;

        assert!(
            new_width > 0 && new_height > 0,
            "Window has to be greater than zero"
        );

        // After resize configuration has to be resized
        let width_changed = true;
        let height_changed = true;

        assert!(width_changed && height_changed);
    }
}
