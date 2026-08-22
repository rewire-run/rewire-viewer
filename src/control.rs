//! Loopback host for Rerun's `ViewerControlService`.
//!
//! The Rerun MCP server (`rerun viewer-mcp`) drives a viewer through the
//! `ViewerControlService` gRPC endpoint. Stock Rerun hosts it next to its message
//! proxy; rewire-viewer is a pure relay client and hosts nothing, so this module
//! spawns the service on a loopback-only endpoint instead. Control commands arrive
//! as UI-command entries on the returned receiver, which the app drains like any
//! other data source. The message proxy that comes with the spawn helper is an
//! implementation detail, not a supported ingestion path.

use std::net::{Ipv4Addr, SocketAddr, TcpListener};

use re_grpc_server::{shutdown, LoopbackServices, MessageProxyHandle, ServerOptions};
use re_log_channel::LogReceiver;

/// Default port for the viewer-control endpoint.
///
/// Port 9876 belongs to the relay, which the viewer dials on the same machine by
/// default, so the control service stays off it.
pub const DEFAULT_PORT: u16 = 9878;

/// Spawns the control service on the loopback interface and returns the proxy
/// handle, which keeps the service alive, plus the receiver carrying its UI
/// commands.
///
/// Prefers [`DEFAULT_PORT`] and falls back to an ephemeral port when it is
/// taken. Must be called from within a tokio runtime.
#[must_use]
pub fn spawn() -> (MessageProxyHandle, LogReceiver) {
    let addr = pick_addr();
    let (rx, proxy) = re_grpc_server::spawn_with_recv_and_services(
        addr,
        ServerOptions::default(),
        shutdown::never(),
        LoopbackServices::default(),
    );
    re_log::info!("Viewer control listening on http://{addr} (loopback only)");
    (proxy, rx)
}

fn pick_addr() -> SocketAddr {
    [DEFAULT_PORT, 0]
        .into_iter()
        .find_map(|port| {
            TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                .and_then(|probe| probe.local_addr())
                .ok()
        })
        .unwrap_or_else(|| SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_PORT)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_addr_prefers_the_default_port() {
        let addr = pick_addr();
        assert!(addr.ip().is_loopback());
        assert!(addr.port() == DEFAULT_PORT || addr.port() != 0);
    }

    #[test]
    fn pick_addr_falls_back_when_the_default_is_taken() {
        let _occupant = TcpListener::bind((Ipv4Addr::LOCALHOST, DEFAULT_PORT));
        let addr = pick_addr();
        assert!(addr.ip().is_loopback());
        assert_ne!(addr.port(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spawn_returns_a_live_handle() {
        let (_proxy, _rx) = spawn();
    }
}
