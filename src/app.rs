#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(target_arch = "wasm32")]
use crate::connection::ConnectionState;
use crate::ui::StatusBar;

/// Top-level eframe application that wraps [`re_viewer::App`] with a Rewire status bar.
pub struct RewireApp {
    rerun_app: re_viewer::App,
    start_time: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    link: crate::connection::RelayLink,
}

impl RewireApp {
    /// Creates a new native viewer observing the given relay link.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(rerun_app: re_viewer::App, link: crate::connection::RelayLink) -> Self {
        Self {
            rerun_app,
            start_time: Instant::now(),
            link,
        }
    }

    /// Creates a new WASM viewer (no relay link).
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

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        self.rerun_app.logic(ctx, frame);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let db = self.rerun_app.recording_db();

        #[cfg(not(target_arch = "wasm32"))]
        let (state, fleet) = (self.link.state(), self.link.fleet());
        #[cfg(target_arch = "wasm32")]
        let (state, fleet) = (ConnectionState::Connecting, None);

        let status = StatusBar::new(db, state, fleet, self.start_time.elapsed());

        egui::Panel::bottom("rewire_status_bar")
            .exact_size(24.0)
            .show(ui, |ui| {
                status.render(ui);
            });

        self.rerun_app.ui(ui, frame);
    }
}
