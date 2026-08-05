//! Client for the RustMine authentication service.
//!
//! Wraps the service's HTTP API with a blocking [`reqwest`] client. The models
//! mirror the service's DTOs with the `Dto` suffix stripped.

mod client;
mod error;
mod models;
