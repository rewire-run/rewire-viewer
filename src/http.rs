use std::sync::{Arc, Mutex};

use rerun::external::{re_log, tokio};
use rewire_extras::HeartbeatTracker;

pub async fn serve(tracker: Arc<Mutex<HeartbeatTracker>>) {
    let app = axum::Router::new()
        .route(
            "/api/info",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "viewer": "rewire",
                    "version": env!("CARGO_PKG_VERSION"),
                }))
            }),
        )
        .route(
            "/api/heartbeat/{id}",
            axum::routing::post({
                let tracker = tracker.clone();
                move |axum::extract::Path(id): axum::extract::Path<String>| {
                    let tracker = tracker.clone();
                    async move {
                        tracker.lock().unwrap().beat(&id);
                        axum::http::StatusCode::NO_CONTENT
                    }
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9877").await.unwrap();
    re_log::info!("Listening for HTTP connections on http://0.0.0.0:9877");
    axum::serve(listener, app).await.unwrap();
}
