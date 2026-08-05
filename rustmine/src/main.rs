use engine::app;

pub mod authentication_api;
pub mod rustmine;
pub mod engine;

fn main() {
    app::window::run().expect("TODO: panic message");
}
