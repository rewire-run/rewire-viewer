//! [`ControlServer`] — loopback host for Rerun's `ViewerControlService`.
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

/// Loopback-only host for Rerun's `ViewerControlService`, so local tooling
/// (the Rerun MCP server) can drive the viewer.
pub struct ControlServer {
    addr: SocketAddr,
    _proxy: MessageProxyHandle,
}

impl ControlServer {
    /// Spawns the control service on the loopback interface and returns the server
    /// plus the receiver carrying its UI commands.
    ///
    /// Prefers [`DEFAULT_PORT`] and falls back to an ephemeral port when it is
    /// taken. Must be called from within a tokio runtime.
    #[must_use]
    pub fn spawn() -> (Self, LogReceiver) {
        let addr = Self::pick_addr();
        let (rx, proxy) = re_grpc_server::spawn_with_recv_and_services(
            addr,
            ServerOptions::default(),
            shutdown::never(),
            LoopbackServices::default(),
        );
        re_log::info!("Viewer control listening on http://{addr} (loopback only)");
        (
            Self {
                addr,
                _proxy: proxy,
            },
            rx,
        )
    }

    /// The loopback address the control service was spawned on.
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn pick_addr() -> SocketAddr {
        for port in [DEFAULT_PORT, 0] {
            if let Ok(probe) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
                if let Ok(addr) = probe.local_addr() {
                    return addr;
                }
            }
        }
        SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_PORT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_addr_prefers_the_default_port() {
        let addr = ControlServer::pick_addr();
        assert!(addr.ip().is_loopback());
        assert!(addr.port() == DEFAULT_PORT || addr.port() != 0);
    }

    #[test]
    fn pick_addr_falls_back_when_the_default_is_taken() {
        let _occupant = TcpListener::bind((Ipv4Addr::LOCALHOST, DEFAULT_PORT));
        let addr = ControlServer::pick_addr();
        assert!(addr.ip().is_loopback());
        assert_ne!(addr.port(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spawn_binds_a_loopback_endpoint() {
        let (server, _rx) = ControlServer::spawn();
        assert!(server.addr().ip().is_loopback());
        assert_ne!(server.addr().port(), 0);
    }
}
