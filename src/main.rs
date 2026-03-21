use std::sync::{Arc, Mutex};

use clap::Parser;
use eframe;
use re_crash_handler;
use re_grpc_server;
use re_memory;
use re_viewer;
use tokio;
use rewire_extras::HeartbeatTracker;
use rewire_viewer::{app, grpc, views};

/// Rewire viewer based on Rerun API for bridge introspection.
#[derive(Parser)]
#[command(name = "rewire-viewer", version)]
struct Cli {}

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Cli::parse();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();
    re_log::setup_logging();
    re_crash_handler::install_crash_handlers(re_viewer::build_info());

    let tracker = Arc::new(Mutex::new(HeartbeatTracker::default()));
    tokio::spawn(grpc::serve(tracker.clone()));

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
            rerun_app.add_view_class::<views::TopicsView>()?;
            rerun_app.add_view_class::<views::NodesView>()?;
            rerun_app.add_view_class::<views::DiagnosticsView>()?;
            rerun_app.add_log_receiver(rx);
            Ok(Box::new(app::RewireApp::new(rerun_app, tracker.clone())))
        }),
    )?;

    Ok(())
}
