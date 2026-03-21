mod app;
mod http;
mod ui;
mod util;
mod views;

use std::sync::{Arc, Mutex};

use rerun::external::{eframe, re_crash_handler, re_grpc_server, re_memory, re_viewer, tokio};
use rewire_extras::HeartbeatTracker;

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();
    rerun::external::re_log::setup_logging();
    re_crash_handler::install_crash_handlers(re_viewer::build_info());

    let tracker = Arc::new(Mutex::new(HeartbeatTracker::default()));
    tokio::spawn(http::serve(tracker.clone()));

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
            rerun_app.add_log_receiver(rx);
            Ok(Box::new(app::RewireApp::new(rerun_app, tracker.clone())))
        }),
    )?;

    Ok(())
}
