use crate::engine::app::account_menu_state::AccountMenuState;
use crate::engine::app::game_state::GameState;
use crate::engine::app::menu_state::MenuState;
use crate::engine::app::multiplayer_menu_state::MultiplayerState;
use crate::engine::app::singleplayer_menu_state::SingleplayerState;
use crate::engine::app::state::{EmptyResult, State, StateResult, TState};
use crate::engine::app::world_creation_state::CreateWorldState;
use crate::rustmine::saves::Save;
use std::sync::Arc;
use wgpu::{Limits, PresentMode};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Icon, Window};

/// The top-level winit application.
///
/// Holds all wgpu resources (surface, adapter, device, queue, config) and the
/// currently active [`TState`]. All fields start as `None` and are populated
/// during [`App::initialize`], which is called once on the first `resumed` event.
pub struct App {
    pub window: Option<Arc<Window>>,
    surface: Option<Arc<wgpu::Surface<'static>>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    state: Option<Box<dyn TState>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates an empty `App`. All fields are `None` until [`initialize`](App::initialize) runs.
    pub fn new() -> Self {
        Self {
            window: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            config: None,
            state: None,
        }
    }

    /// Instantiates the [`TState`] implementation that corresponds to `state` and
    /// replaces the current active state. `None` is a no-op; [`State::Exit`] exits
    /// the event loop.
    fn switch_state(&mut self, event_loop: &ActiveEventLoop, state: Option<State>) {
        let window = self.window.clone().unwrap();

        if let Some(st) = &state {
            log::info!("Switching state to {:?}", st);
        }

        match state {
            None => {}
            Some(State::Exit) => event_loop.exit(),
            Some(State::MainMenu) => {
                self.state = Some(Box::new(
                    pollster::block_on(MenuState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                    ))
                    .unwrap(),
                ))
            }
            Some(State::AccountMenu) => {
                self.state = Some(Box::new(
                    pollster::block_on(AccountMenuState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                    ))
                    .unwrap(),
                ))
            }
            Some(State::Game(world_name)) => {
                self.state = Some(Box::new(
                    pollster::block_on(GameState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.adapter.clone().unwrap(),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                        world_name,
                    ))
                    .unwrap(),
                ))
            }
            Some(State::SingleplayerMenu) => {
                self.state = Some(Box::new(
                    pollster::block_on(SingleplayerState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                    ))
                    .unwrap(),
                ))
            }
            Some(State::CreateWorldMenu) => {
                self.state = Some(Box::new(
                    pollster::block_on(CreateWorldState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                    ))
                    .unwrap(),
                ))
            }
            Some(State::MultiplayerMenu) => {
                self.state = Some(Box::new(
                    pollster::block_on(MultiplayerState::new(
                        window,
                        Arc::clone(self.surface.as_ref().unwrap()),
                        self.device.clone().unwrap(),
                        self.queue.clone().unwrap(),
                        self.config.clone().unwrap(),
                    ))
                    .unwrap(),
                ))
            }
        }
    }

    /// Exits the event loop on [`Err`], otherwise does nothing.
    fn handle_empty_result(&mut self, event_loop: &ActiveEventLoop, result: EmptyResult) {
        if let Err(e) = result {
            log::error!("Error while state update: {}", e);
            event_loop.exit();
        }
    }

    /// Exits the event loop on [`Err`]; on `Ok(Some(state))` delegates
    /// to [`switch_state`](App::switch_state).
    fn handle_state_result(&mut self, event_loop: &ActiveEventLoop, result: StateResult) {
        match result {
            Err(e) => {
                log::error!("Error while state update: {}", e);
                event_loop.exit();
            }
            Ok(state) => self.switch_state(event_loop, state),
        }
    }

    /// Creates the window, wgpu instance/surface/adapter/device/queue/config,
    /// then transitions to [`State::MainMenu`].
    ///
    /// The surface format prefers an sRGB format if available.
    /// VSync is disabled (`AutoNoVsync`). Max buffer size is set to 1 GiB.
    async fn initialize(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        log::debug!("Initializing window");
        let icon_rgba = include_bytes!("../../assets/icons/rustmine-icon.png");
        let image = image::load_from_memory(icon_rgba)?.into_rgba8();
        let (icon_width, icon_height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), icon_width, icon_height)?;

        let window_attributes = Window::default_attributes()
            .with_title("RustMine")
            .with_window_icon(Some(icon.clone()))
            .with_maximized(true);

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        let size = window.inner_size();

        log::debug!("Initializing WGPU");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        log::debug!("Initializing WGPU surface");
        let surface = instance.create_surface(window.clone())?;

        log::debug!("Initializing WGPU device");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let info = adapter.get_info();
        log::info!("Using GPU: {} {}", info.name, info.vendor);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: Limits {
                    max_buffer_size: 1024 << 20,
                    ..Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
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
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        self.window = Some(window.clone());
        self.surface = Some(Arc::new(surface));
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);

        log::debug!("Initializing initial state");
        self.switch_state(event_loop, Some(State::MainMenu));
        Ok(())
    }
}

impl ApplicationHandler for App {
    /// Calls [`initialize`](App::initialize) on the first resume; exits on failure.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = pollster::block_on(self.initialize(event_loop)) {
            log::error!("Failed to initialize: {}", e);
            event_loop.exit();
        }
    }

    /// Dispatches winit window events to the active state:
    /// - `CloseRequested` → [`State::Exit`]
    /// - `Resized` → [`TState::handle_resize`]
    /// - `RedrawRequested` → [`TState::update`] then [`TState::render`]
    /// - `MouseInput` → [`TState::handle_mouse_button_press`]
    /// - `KeyboardInput` → [`TState::handle_key_press`]
    /// - All events → [`TState::handle_window_event`]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        state
            .handle_window_event(&event)
            .expect("TODO: panic message");

        match event {
            WindowEvent::CloseRequested => self.switch_state(event_loop, Some(State::Exit)),
            WindowEvent::Resized(size) => {
                let res = state.handle_resize(size.width, size.height);
                self.handle_empty_result(event_loop, res);
            }
            WindowEvent::RedrawRequested => {
                let update_res = state.update();
                let render_res = state.render();
                self.handle_state_result(event_loop, update_res);
                self.handle_empty_result(event_loop, render_res);
            }
            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                let res = state.handle_mouse_button_press(button, btn_state.is_pressed());
                self.handle_state_result(event_loop, res);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let res = state.handle_key_press(code, key_state.is_pressed());
                self.handle_state_result(event_loop, res);
            }
            _ => {}
        }
    }

    /// Forwards `MouseMotion` deltas to [`TState::handle_mouse_movement`].
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let res = state.handle_mouse_movement(dx, dy);
            self.handle_empty_result(_event_loop, res);
        }
    }
}

/// Initializes logging, creates the saves directory, and runs the winit event loop.
pub(crate) fn run() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    log::info!("Creating saves directory");
    Save::create_dir()?;

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(app.state.is_none(), "State should be None at the start");
    }

    #[test]
    fn test_window_attributes() {
        let title = "RustMine";
        let width: f64 = 1280.0;
        let height: f64 = 720.0;

        let attr = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));

        assert_eq!(attr.title, title);
        if let Some(winit::dpi::Size::Logical(size)) = attr.inner_size {
            assert_eq!(size.width, width);
            assert_eq!(size.height, height);
        }
    }
}
