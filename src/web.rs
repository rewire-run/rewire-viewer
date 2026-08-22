use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::{app::RewireApp, theme::preference as theme_preference, views};

/// Handle for the WASM viewer, exposed to JavaScript via `wasm-bindgen`.
#[wasm_bindgen]
pub struct RewireWebHandle {
    runner: eframe::WebRunner,
    /// Captured once the app starts, so the theme can be changed from JS
    /// without reloading the viewer (and re-downloading the recording).
    egui_ctx: Rc<RefCell<Option<egui::Context>>>,
}

#[wasm_bindgen]
impl RewireWebHandle {
    /// Creates a new viewer handle and initializes logging.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RewireWebHandle, JsValue> {
        eframe::WebLogger::init(re_log::external::log::LevelFilter::Debug).ok();
        Ok(Self {
            runner: eframe::WebRunner::new(),
            egui_ctx: Rc::new(RefCell::new(None)),
        })
    }

    /// Starts the viewer on the given canvas, optionally loading an `.rrd` URL
    /// and forcing a theme (`dark`, `light` or `system`).
    ///
    /// Without `theme` the viewer follows the browser's `prefers-color-scheme`,
    /// which an embedding page cannot influence: `color-scheme` does not cross
    /// an origin boundary, and the UI is painted into a canvas rather than out
    /// of browser widgets. Passing the parameter is the only way for a host
    /// page to pin the embed.
    #[wasm_bindgen]
    pub async fn start(
        &self,
        canvas: web_sys::HtmlCanvasElement,
        url: Option<String>,
        theme: Option<String>,
    ) -> Result<(), JsValue> {
        let web_options = eframe::WebOptions::default();
        let ctx_slot = Rc::clone(&self.egui_ctx);

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

                    // After `App::new`, not before: on a browser with no
                    // persisted version key it resets the preference to
                    // `System` as part of its pre-0.24 upgrade path, which
                    // would silently undo the requested theme.
                    if let Some(preference) = theme.as_deref().and_then(theme_preference) {
                        cc.egui_ctx.set_theme(preference);
                    }
                    *ctx_slot.borrow_mut() = Some(cc.egui_ctx.clone());

                    Ok(Box::new(RewireApp::new(rerun_app)))
                }),
            )
            .await
    }

    /// Changes the theme of a running viewer — `dark`, `light` or `system`.
    ///
    /// Lets a host page follow its own theme toggle without reloading the
    /// iframe, which would re-download the recording. No-op before `start`.
    #[wasm_bindgen]
    pub fn set_theme(&self, theme: &str) {
        let Some(preference) = theme_preference(theme) else {
            return;
        };
        if let Some(ctx) = self.egui_ctx.borrow().as_ref() {
            ctx.set_theme(preference);
            ctx.request_repaint();
        }
    }

    /// Destroys the viewer and releases GPU resources.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
        self.egui_ctx.borrow_mut().take();
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
