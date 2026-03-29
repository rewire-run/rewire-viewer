use std::sync::{Arc, Mutex};

use clap::Parser;
use rewire_extras::HeartbeatTracker;
use rewire_viewer::{app, grpc, views};

/// Rewire viewer based on Rerun API for bridge introspection.
#[derive(Parser)]
#[command(name = "rewire-viewer", version)]
struct Cli {
    /// Port for the Rerun gRPC data stream
    #[arg(long, default_value_t = 9876)]
    port: u16,

    /// Port for the Rewire gRPC service (bridge heartbeats, viewer info) [default: port + 1]
    #[arg(long)]
    grpc_port: Option<u16>,
}

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run(cli))
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();
    re_log::setup_logging();
    re_crash_handler::install_crash_handlers(re_viewer::build_info());

    let grpc_port = cli.grpc_port.unwrap_or(cli.port + 1);

    for port in [cli.port, grpc_port] {
        if std::net::TcpListener::bind(("0.0.0.0", port)).is_err() {
            re_log::error!(
                "Port {port} is already in use. Is another viewer or Rerun instance running?"
            );
            std::process::exit(1);
        }
    }

    let tracker = Arc::new(Mutex::new(HeartbeatTracker::default()));
    tokio::spawn(grpc::serve(tracker.clone(), grpc_port));

    let addr = format!("0.0.0.0:{}", cli.port);
    let rx = re_grpc_server::spawn_with_recv(
        addr.parse()?,
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
