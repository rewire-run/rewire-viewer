//! A custom [Rerun](https://rerun.io) viewer for ROS 2 visualization, built on Rerun v0.31.
//!
//! Rewire Viewer extends the Rerun native viewer with ROS 2-specific panels (Topics, Nodes,
//! Diagnostics) and a status bar showing bridge connectivity. It runs a gRPC server that bridges
//! use to advertise themselves and send heartbeats.
//!
//! Supports both native (desktop) and WebAssembly builds.

/// Wrapper around [`re_viewer::App`] that adds the Rewire status bar.
pub mod app;
/// gRPC service for bridge heartbeats and viewer info queries.
#[cfg(not(target_arch = "wasm32"))]
pub mod grpc;
/// Custom view icons for Rewire panels.
pub mod icons;
/// Status bar and shared UI helpers.
pub mod ui;
/// Shared utilities for extracting data from Arrow arrays.
pub mod util;
/// Custom Rerun SpaceView classes for ROS 2 data.
pub mod views;

#[cfg(target_arch = "wasm32")]
mod web;

/// Entry point for the WebAssembly build.
#[cfg(target_arch = "wasm32")]
pub use web::RewireWebHandle;
