use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum State {
    Exit,
    MainMenu,
    AccountMenu,
    SingleplayerMenu,
    CreateWorldMenu,
    MultiplayerMenu,
    Game(String),
}

pub type EmptyResult = anyhow::Result<()>;
pub type StateResult = anyhow::Result<Option<State>>;

pub trait TState {
    fn handle_resize(&mut self, width: u32, height: u32) -> EmptyResult;
    fn handle_key_press(&mut self, key: KeyCode, pressed: bool) -> StateResult;
    fn handle_mouse_button_press(&mut self, button: MouseButton, pressed: bool) -> StateResult;

    /// Called on mouse movement. `dx`/`dy` are the delta in pixels since the last event.
    fn handle_mouse_movement(&mut self, dx: f64, dy: f64) -> EmptyResult;

    /// Called for any winit window event not covered by the other handlers.
    fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> EmptyResult;

    /// Advances the state's logic by one tick. Called once per frame before `render`.
    fn update(&mut self) -> StateResult;

    /// Draws the current frame.
    fn render(&mut self) -> EmptyResult;
}
