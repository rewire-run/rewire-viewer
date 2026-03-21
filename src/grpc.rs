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
        self.tracker
            .lock()
            .unwrap()
            .beat(&req.into_inner().bridge_id);
        Ok(Response::new(HeartbeatResponse {}))
    }
}

pub async fn serve(tracker: Arc<Mutex<HeartbeatTracker>>) {
    let svc = RewireServiceImpl { tracker };
    let addr = "0.0.0.0:9877".parse().unwrap();
    re_log::info!("Listening for gRPC connections on 0.0.0.0:9877. Used by bridges to advertise themselves and query viewer info.");
    tonic::transport::Server::builder()
        .add_service(RewireServiceServer::new(svc))
        .serve(addr)
        .await
        .unwrap();
}
