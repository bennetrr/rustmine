use engine::app;

pub mod authentication_api;
pub mod engine;
pub mod rustmine;

fn main() {
    app::window::run().expect("TODO: panic message");
}
