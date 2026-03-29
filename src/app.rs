#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use eframe;
use egui;
use re_viewer;

use crate::ui::StatusBar;

/// Top-level eframe application that wraps [`re_viewer::App`] with a Rewire status bar.
pub struct RewireApp {
    rerun_app: re_viewer::App,
    start_time: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    tracker: std::sync::Arc<std::sync::Mutex<rewire_extras::HeartbeatTracker>>,
}

impl RewireApp {
    /// Creates a new native viewer with a heartbeat tracker for bridge connectivity.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        rerun_app: re_viewer::App,
        tracker: std::sync::Arc<std::sync::Mutex<rewire_extras::HeartbeatTracker>>,
    ) -> Self {
        Self {
            rerun_app,
            start_time: Instant::now(),
            tracker,
        }
    }

    /// Creates a new WASM viewer (no heartbeat tracking).
    #[cfg(target_arch = "wasm32")]
    pub fn new(rerun_app: re_viewer::App) -> Self {
        Self {
            rerun_app,
            start_time: Instant::now(),
        }
    }
}

impl eframe::App for RewireApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.rerun_app.save(storage);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let db = self.rerun_app.recording_db();

        #[cfg(not(target_arch = "wasm32"))]
        let (connected, bridge_count) = self.tracker.lock().unwrap().status();
        #[cfg(target_arch = "wasm32")]
        let (connected, bridge_count) = (false, 0);

        let status = StatusBar::new(db, connected, bridge_count, self.start_time.elapsed());

        egui::TopBottomPanel::bottom("rewire_status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                status.render(ui);
            });

        self.rerun_app.update(ctx, frame);
    }
}
