//! A custom [Rerun](https://rerun.io) viewer for ROS 2 visualization, built on Rerun v0.35.
//!
//! Rewire Viewer extends the Rerun native viewer with ROS 2-specific panels (Topics, Nodes,
//! Diagnostics) and a status bar showing relay connectivity. It is a pure client: it dials a
//! relay (`rewire serve`) or any Rerun gRPC endpoint for data and auto-reconnects with backoff.
//!
//! Supports both native (desktop) and WebAssembly builds.

/// Wrapper around [`re_viewer::App`] that adds the Rewire status bar.
pub mod app;
/// Reconnecting client connection to a relay.
pub mod connection;
/// Loopback viewer-control service for the Rerun MCP.
#[cfg(not(target_arch = "wasm32"))]
pub mod control;
/// Custom view icons for Rewire panels.
pub mod icons;
/// Parsing for the viewer's theme preference.
pub mod theme;
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
