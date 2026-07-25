use clap::Parser;
use rewire_viewer::connection::RelayLink;
use rewire_viewer::control::ControlServer;
use rewire_viewer::{app, views};

/// Rewire viewer based on Rerun API for bridge introspection.
#[derive(Parser)]
#[command(name = "rewire-viewer", version)]
struct Cli {
    /// Relay endpoint to connect to (`host`, `host:port`, or `rerun+http://host:port/proxy`)
    #[arg(long, default_value = "127.0.0.1:9876")]
    connect: String,
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

    let uri: re_uri::ProxyUri = RelayLink::normalize(&cli.connect).parse()?;
    re_log::info!("Connecting to {uri}");
    let (link, rx) = RelayLink::open(uri);
    let (_control_server, control_rx) = ControlServer::spawn();

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
            rerun_app.add_log_receiver(control_rx);
            Ok(Box::new(app::RewireApp::new(rerun_app, link)))
        }),
    )?;

    Ok(())
}
