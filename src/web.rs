use wasm_bindgen::prelude::*;

use eframe;
use re_viewer;

use crate::{app::RewireApp, views};

#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebHandle, JsValue> {
        eframe::WebLogger::init(re_log::LevelFilter::Debug).ok();
        Ok(Self {
            runner: eframe::WebRunner::new(),
        })
    }

    #[wasm_bindgen]
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
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
                    Ok(Box::new(RewireApp::new(rerun_app)))
                }),
            )
            .await
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.panic_summary().is_some()
    }

    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }
}
