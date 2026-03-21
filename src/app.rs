use std::sync::{Arc, Mutex};
use std::time::Instant;

use rerun::external::{eframe, egui, re_viewer};
use rewire_extras::HeartbeatTracker;

use crate::ui::StatusBar;

pub struct RewireApp {
    rerun_app: re_viewer::App,
    start_time: Instant,
    tracker: Arc<Mutex<HeartbeatTracker>>,
}

impl RewireApp {
    pub fn new(
        rerun_app: re_viewer::App,
        tracker: Arc<Mutex<HeartbeatTracker>>,
    ) -> Self {
        Self {
            rerun_app,
            start_time: Instant::now(),
            tracker,
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
        let (connected, bridge_count) = self.tracker.lock().unwrap().status();
        let status = StatusBar::new(db, connected, bridge_count, self.start_time.elapsed());

        egui::TopBottomPanel::bottom("rewire_status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                status.render(ui);
            });

        self.rerun_app.update(ctx, frame);
    }
}
