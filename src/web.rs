use wasm_bindgen::prelude::*;

use eframe;
use re_viewer;

use crate::{app::RewireApp, views};

/// Handle for the WASM viewer, exposed to JavaScript via `wasm-bindgen`.
#[wasm_bindgen]
pub struct RewireWebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl RewireWebHandle {
    /// Creates a new viewer handle and initializes logging.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RewireWebHandle, JsValue> {
        eframe::WebLogger::init(re_log::LevelFilter::Debug).ok();
        Ok(Self {
            runner: eframe::WebRunner::new(),
        })
    }

    /// Starts the viewer on the given canvas, optionally loading an `.rrd` URL.
    #[wasm_bindgen]
    pub async fn start(
        &self,
        canvas: web_sys::HtmlCanvasElement,
        url: Option<String>,
    ) -> Result<(), JsValue> {
        let web_options = eframe::WebOptions::default();

        self.runner
            .start(
                canvas,
                web_options,
                Box::new(move |cc| {
                    re_viewer::customize_eframe_and_setup_renderer(cc)?;
                    let mut rerun_app = re_viewer::App::new(
                        re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread(),
                        re_viewer::build_info(),
                        re_viewer::AppEnvironment::Custom("Rewire Viewer".to_owned()),
                        re_viewer::StartupOptions::default(),
                        cc,
                        None,
                        re_viewer::AsyncRuntimeHandle::from_current_tokio_runtime_or_wasmbindgen()?,
                    );
                    rerun_app.add_view_class::<views::TopicsView>()?;
                    rerun_app.add_view_class::<views::NodesView>()?;
                    rerun_app.add_view_class::<views::DiagnosticsView>()?;

                    if let Some(rrd_url) = &url {
                        rerun_app.open_url_or_file(rrd_url);
                    }

                    Ok(Box::new(RewireApp::new(rerun_app)))
                }),
            )
            .await
    }

    /// Destroys the viewer and releases GPU resources.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// Returns `true` if the viewer has panicked.
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.panic_summary().is_some()
    }

    /// Returns the panic message, if any.
    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }
}
