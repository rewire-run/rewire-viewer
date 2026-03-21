mod nodes_system;
mod nodes_view;
mod topics_system;
mod view_class;

use rerun::external::{
    arrow, eframe, egui, re_chunk_store, re_crash_handler, re_entity_db, re_grpc_server, re_log,
    re_log_types, re_memory, re_viewer, tokio,
};
use rewire_extras::{ROS2NodeInfo, ROS2TopicInfo};

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
            rerun_app.add_view_class::<view_class::TopicsView>()?;
            rerun_app.add_view_class::<nodes_view::NodesView>()?;
            rerun_app.add_log_receiver(rx);
            Ok(Box::new(RewireApp {
                rerun_app,
                start_time: std::time::Instant::now(),
            }))
        }),
    )?;

    Ok(())
}

struct RewireApp {
    rerun_app: re_viewer::App,
    start_time: std::time::Instant,
}

impl eframe::App for RewireApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.rerun_app.save(storage);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let db = self.rerun_app.recording_db();
        let (connected, bridge_count) = db.map(check_heartbeats).unwrap_or((false, 0));
        let status = StatusBarState {
            has_db: db.is_some(),
            connected,
            bridge_count,
            node_count: db.map(node_count).unwrap_or(0),
            topic_count: db.map(topic_count).unwrap_or(0),
            app_id: db
                .and_then(|db| db.store_info().map(|i| i.application_id().to_string()))
                .unwrap_or_default(),
            uptime: self.start_time.elapsed(),
        };

        egui::TopBottomPanel::bottom("rewire_status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                status_bar(ui, &status);
            });

        self.rerun_app.update(ctx, frame);
    }
}

struct StatusBarState {
    has_db: bool,
    connected: bool,
    bridge_count: usize,
    node_count: usize,
    topic_count: usize,
    app_id: String,
    uptime: std::time::Duration,
}

fn status_bar(ui: &mut egui::Ui, s: &StatusBarState) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.add_space(8.0);

        if !s.has_db {
            ui.colored_label(egui::Color32::GRAY, "⬤");
            ui.label("Waiting for connection...");
            return;
        }

        if s.connected {
            ui.colored_label(egui::Color32::from_rgb(80, 200, 120), "⬤");
            ui.label("Connected");
        } else {
            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "⬤");
            ui.label("Disconnected");
        }

        ui.separator();

        let suffix = if s.bridge_count == 1 { "" } else { "s" };
        ui.label(format!("{} bridge{suffix}", s.bridge_count));
        ui.separator();

        if !s.app_id.is_empty() {
            ui.label(format!("App: {}", s.app_id));
            ui.separator();
        }

        let node_suffix = if s.node_count == 1 { "" } else { "s" };
        ui.label(format!("{} node{node_suffix}", s.node_count));
        ui.separator();

        ui.label(format!("{} topics", s.topic_count));
        ui.separator();

        let secs = s.uptime.as_secs();
        let mins = secs / 60;
        let hours = mins / 60;
        if hours > 0 {
            ui.label(format!("{}h {}m", hours, mins % 60));
        } else if mins > 0 {
            ui.label(format!("{}m {}s", mins, secs % 60));
        } else {
            ui.label(format!("{}s", secs));
        }
    });
}

const HEARTBEAT_STALENESS_SECS: i64 = 5;

fn check_heartbeats(entity_db: &re_entity_db::EntityDb) -> (bool, usize) {
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;
    let timeline = re_log_types::TimelineName::log_time();
    let scalars_id = rerun::Scalars::descriptor_scalars().component;

    let paths: Vec<re_log_types::EntityPath> =
        entity_db.sorted_entity_paths().cloned().collect();

    let alive = paths
        .iter()
        .filter(|p| {
            let s = format!("{p}");
            s.starts_with("/rewire/bridge/") && s.ends_with("/heartbeat")
        })
        .filter(|p| {
            let query = re_chunk_store::LatestAtQuery::latest(timeline);
            let results = entity_db
                .storage_engine()
                .cache()
                .latest_at(&query, p, [scalars_id]);

            if results.is_empty() {
                return false;
            }

            let (time, _) = results.max_index();
            let nanos = time.as_i64();
            nanos > 0
                && now_nanos > nanos
                && (now_nanos - nanos) / 1_000_000_000 < HEARTBEAT_STALENESS_SECS
        })
        .count();

    (alive > 0, alive)
}

fn node_count(entity_db: &re_entity_db::EntityDb) -> usize {
    let timeline = re_log_types::TimelineName::log_time();
    let query = re_chunk_store::LatestAtQuery::latest(timeline);
    let path = re_log_types::EntityPath::from("/rewire/nodes");
    let id = ROS2NodeInfo::descriptor_node_name().component;

    entity_db
        .storage_engine()
        .cache()
        .latest_at(&query, &path, [id])
        .component_batch_raw(id)
        .map(|arr| {
            use arrow::array::Array as _;
            arr.len()
        })
        .unwrap_or(0)
}

fn topic_count(entity_db: &re_entity_db::EntityDb) -> usize {
    let timeline = re_log_types::TimelineName::log_time();
    let query = re_chunk_store::LatestAtQuery::latest(timeline);
    let path = re_log_types::EntityPath::from("/rewire/topics");
    let id = ROS2TopicInfo::descriptor_topic_name().component;

    entity_db
        .storage_engine()
        .cache()
        .latest_at(&query, &path, [id])
        .component_batch_raw(id)
        .map(|arr| {
            use arrow::array::Array as _;
            arr.len()
        })
        .unwrap_or(0)
}
