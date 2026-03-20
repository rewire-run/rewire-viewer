use rerun::external::{
    eframe, egui, re_crash_handler, re_entity_db, re_grpc_server, re_log, re_log_types, re_memory,
    re_viewer, tokio,
};

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();
    re_log::setup_logging();
    re_crash_handler::install_crash_handlers(re_viewer::build_info());

    let rx = re_grpc_server::spawn_with_recv(
        "0.0.0.0:9876".parse()?,
        Default::default(),
        re_grpc_server::shutdown::never(),
    );

    let mut native_options = re_viewer::native::eframe_options(None);
    native_options.viewport = native_options.viewport.with_app_id("rewire_viewer");

    let startup_options = re_viewer::StartupOptions::default();
    let app_env = re_viewer::AppEnvironment::Custom("Rewire Viewer".to_owned());

    eframe::run_native(
        "Rewire Viewer",
        native_options,
        Box::new(move |cc| {
            re_viewer::customize_eframe_and_setup_renderer(cc)?;
            let mut rerun_app = re_viewer::App::new(
                main_thread_token,
                re_viewer::build_info(),
                app_env,
                startup_options,
                cc,
                None,
                re_viewer::AsyncRuntimeHandle::from_current_tokio_runtime_or_wasmbindgen()?,
            );
            rerun_app.add_log_receiver(rx);
            Ok(Box::new(RewireApp { rerun_app }))
        }),
    )?;

    Ok(())
}

struct RewireApp {
    rerun_app: re_viewer::App,
}

impl eframe::App for RewireApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.rerun_app.save(storage);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::right("rewire_topics")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.topics_panel(ui);
            });

        self.rerun_app.update(ctx, frame);
    }
}

impl RewireApp {
    fn topics_panel(&self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            ui.strong("Topics");
        });
        ui.separator();

        let Some(entity_db) = self.rerun_app.recording_db() else {
            ui.label("Waiting for data...");
            return;
        };

        topics_ui(ui, entity_db);
    }
}

fn topics_ui(ui: &mut egui::Ui, entity_db: &re_entity_db::EntityDb) {
    let timeline = re_log_types::TimelineName::log_time();

    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for entity_path in entity_db.sorted_entity_paths() {
                let path_str = entity_path.to_string();
                if path_str.starts_with("/rewire") || path_str.starts_with("/bridge") {
                    continue;
                }

                let components: Vec<String> = entity_db
                    .storage_engine()
                    .store()
                    .all_components_on_timeline_sorted(&timeline, &entity_path)
                    .map(|c| c.iter().map(|c| c.to_string()).collect())
                    .unwrap_or_default();

                ui.collapsing(&path_str, |ui| {
                    for component in &components {
                        ui.label(component);
                    }
                });
            }
        });
}
