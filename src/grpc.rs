use std::sync::{Arc, Mutex};

use re_log;
use rewire_extras::proto::v1::rewire_service_server::{RewireService, RewireServiceServer};
use rewire_extras::proto::v1::{
    GetInfoRequest, GetInfoResponse, HeartbeatRequest, HeartbeatResponse,
};
use rewire_extras::HeartbeatTracker;
use tonic::{Request, Response, Status};

struct RewireServiceImpl {
    tracker: Arc<Mutex<HeartbeatTracker>>,
}

#[tonic::async_trait]
impl RewireService for RewireServiceImpl {
    async fn get_info(
        &self,
        _req: Request<GetInfoRequest>,
    ) -> Result<Response<GetInfoResponse>, Status> {
        Ok(Response::new(GetInfoResponse {
            viewer: "rewire".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }))
    }

    async fn heartbeat(
        &self,
        req: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let inner = req.into_inner();
        let state = rewire_extras::BridgeState::from(inner.state);
        self.tracker.lock().unwrap().beat(&inner.bridge_id, state);
        Ok(Response::new(HeartbeatResponse {}))
    }
}

/// Starts the Rewire gRPC service on the given port.
///
/// Serves [`RewireService`] which handles bridge heartbeats and viewer info queries.
/// Exits the process if the server fails to bind.
pub async fn serve(tracker: Arc<Mutex<HeartbeatTracker>>, port: u16) {
    let svc = RewireServiceImpl { tracker };
    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();
    re_log::info!("Listening for gRPC connections on {addr}. Used by bridges to advertise themselves and query viewer info.");
    if let Err(err) = tonic::transport::Server::builder()
        .add_service(RewireServiceServer::new(svc))
        .serve(addr)
        .await
    {
        re_log::error!(
            "Rewire gRPC server on port {port} failed: {err}. Is another viewer already running?"
        );
        std::process::exit(1);
    }
}
