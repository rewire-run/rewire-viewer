//! Self-healing client connection to a relay or any Rerun `MessageProxyService` host.
//!
//! Rerun's `re_grpc_client::read::stream` is one-shot: its receiver dies on the first
//! error or close and never redials. [`RelayLink`] hands the viewer one stable receiver
//! and feeds it from a pump that redials with backoff, exposing a [`ConnectionState`]
//! for the status bar.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex, PoisonError};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use re_log_channel::{LogReceiver, LogSender, LogSource, SmartMessagePayload};
#[cfg(not(target_arch = "wasm32"))]
use rewire_extras::proto::v3::relay_service_client::RelayServiceClient;
#[cfg(not(target_arch = "wasm32"))]
use rewire_extras::proto::v3::GetHeartbeatsRequest;
#[cfg(not(target_arch = "wasm32"))]
use rewire_extras::BridgeState;

/// Where the link to the relay currently stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Dialing; nothing has been received on this link yet.
    Connecting,
    /// The current stream is live and has delivered messages.
    Connected,
    /// A live stream dropped; redialing with backoff.
    Reconnecting,
}

/// Snapshot of the relay's bridge fleet, refreshed by the poll task.
#[derive(Debug, Clone)]
pub struct FleetSnapshot {
    /// Number of bridges the relay has heard from recently.
    pub bridge_count: usize,
    /// True when at least one bridge reports itself actively streaming.
    pub any_active: bool,
}

#[cfg(not(target_arch = "wasm32"))]
const BACKOFF_MIN: Duration = Duration::from_millis(250);
#[cfg(not(target_arch = "wasm32"))]
const BACKOFF_MAX: Duration = Duration::from_secs(5);
#[cfg(not(target_arch = "wasm32"))]
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[cfg(not(target_arch = "wasm32"))]
enum Forward {
    StreamEnded { delivered: bool },
    ReceiverClosed,
}

/// Reconnecting link to a relay, owning the redial loop behind a stable receiver.
#[cfg(not(target_arch = "wasm32"))]
pub struct RelayLink {
    state: Arc<Mutex<ConnectionState>>,
    fleet: Arc<Mutex<Option<FleetSnapshot>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl RelayLink {
    /// Expands shorthand (`host:port`, `host`) into a full
    /// `rerun+http://host:port/proxy` URI, replacing `localhost` with `127.0.0.1`.
    #[must_use]
    pub fn normalize(url: &str) -> String {
        let url = url.replace("localhost", "127.0.0.1");
        if url.contains("://") {
            if url.contains("/proxy") {
                url
            } else {
                format!("{url}/proxy")
            }
        } else {
            format!("rerun+http://{url}/proxy")
        }
    }

    /// Opens the link and returns the stable receiver to hand to the viewer once.
    ///
    /// Must be called from within a tokio runtime; the pump is spawned on it.
    #[must_use]
    pub fn open(uri: re_uri::ProxyUri) -> (Self, LogReceiver) {
        let (tx, rx) = re_log_channel::log_channel(LogSource::MessageProxy(uri.clone()));
        let state = Arc::new(Mutex::new(ConnectionState::Connecting));
        let fleet = Arc::new(Mutex::new(None));
        tokio::spawn(Self::poll(uri.origin.as_url(), Arc::clone(&fleet)));
        tokio::spawn(Self::run(uri, tx, Arc::clone(&state)));
        (Self { state, fleet }, rx)
    }

    /// Current state of the link, for the status bar.
    #[must_use]
    pub fn state(&self) -> ConnectionState {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Latest fleet snapshot from the relay, or `None` while the poll is failing.
    #[must_use]
    pub fn fleet(&self) -> Option<FleetSnapshot> {
        self.fleet
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    async fn poll(url: String, fleet: Arc<Mutex<Option<FleetSnapshot>>>) {
        loop {
            let mut client = match RelayServiceClient::connect(url.clone()).await {
                Ok(client) => client,
                Err(err) => {
                    re_log::debug!("Fleet poll connect failed: {err}");
                    Self::store_fleet(&fleet, None);
                    tokio::time::sleep(POLL_INTERVAL).await;
                    continue;
                }
            };
            loop {
                let response = match client.get_heartbeats(GetHeartbeatsRequest {}).await {
                    Ok(response) => response,
                    Err(err) => {
                        re_log::debug!("Fleet poll request failed: {err}");
                        break;
                    }
                };
                let bridges = response.into_inner().bridges;
                let snapshot = FleetSnapshot {
                    bridge_count: bridges.len(),
                    any_active: bridges
                        .iter()
                        .any(|b| matches!(BridgeState::from(b.state), BridgeState::Active)),
                };
                Self::store_fleet(&fleet, Some(snapshot));
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Self::store_fleet(&fleet, None);
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    fn store_fleet(fleet: &Mutex<Option<FleetSnapshot>>, value: Option<FleetSnapshot>) {
        let mut fleet = fleet.lock().unwrap_or_else(PoisonError::into_inner);
        match (fleet.is_some(), value.is_some()) {
            (false, true) => re_log::debug!("Fleet poll connected"),
            (true, false) => re_log::debug!("Fleet poll lost"),
            _ => {}
        }
        *fleet = value;
    }

    async fn run(uri: re_uri::ProxyUri, tx: LogSender, state: Arc<Mutex<ConnectionState>>) {
        let mut backoff = BACKOFF_MIN;
        loop {
            let inner = re_grpc_client::read::stream(uri.clone());
            let session_tx = tx.clone();
            let session_state = Arc::clone(&state);
            let outcome = tokio::task::spawn_blocking(move || {
                Self::forward(&inner, &session_tx, &session_state)
            })
            .await;

            match outcome {
                Ok(Forward::StreamEnded { delivered }) => {
                    if delivered {
                        backoff = BACKOFF_MIN;
                        Self::set(&state, ConnectionState::Reconnecting);
                    }
                }
                Ok(Forward::ReceiverClosed) | Err(_) => return,
            }

            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(BACKOFF_MAX);
        }
    }

    fn forward(inner: &LogReceiver, tx: &LogSender, state: &Mutex<ConnectionState>) -> Forward {
        let mut delivered = false;
        loop {
            let Ok(msg) = inner.recv() else {
                return Forward::StreamEnded { delivered };
            };
            match msg.payload {
                SmartMessagePayload::Msg(data) => {
                    if tx.send(data).is_err() {
                        return Forward::ReceiverClosed;
                    }
                    if !delivered {
                        delivered = true;
                        Self::set(state, ConnectionState::Connected);
                    }
                }
                SmartMessagePayload::Flush { on_flush_done } => on_flush_done(),
                SmartMessagePayload::Quit(err) => {
                    if let Some(err) = err {
                        re_log::debug!("Relay stream ended: {err}");
                    }
                    return Forward::StreamEnded { delivered };
                }
            }
        }
    }

    fn set(state: &Mutex<ConnectionState>, value: ConnectionState) {
        match value {
            ConnectionState::Connected => re_log::info!("Relay link established"),
            ConnectionState::Reconnecting => re_log::warn!("Relay link lost, reconnecting"),
            ConnectionState::Connecting => {}
        }
        *state.lock().unwrap_or_else(PoisonError::into_inner) = value;
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    use re_log_channel::DataSourceMessage;
    use re_log_types::{StoreId, StoreKind};

    fn channel() -> (LogSender, LogReceiver) {
        let uri = "rerun+http://127.0.0.1:9876/proxy".parse().unwrap();
        re_log_channel::log_channel(LogSource::MessageProxy(uri))
    }

    fn message() -> DataSourceMessage {
        DataSourceMessage::RrdManifestComplete(StoreId::random(StoreKind::Recording, "test"))
    }

    #[test]
    fn normalize_expands_shorthand() {
        assert_eq!(
            RelayLink::normalize("127.0.0.1:9876"),
            "rerun+http://127.0.0.1:9876/proxy"
        );
        assert_eq!(
            RelayLink::normalize("localhost:4321"),
            "rerun+http://127.0.0.1:4321/proxy"
        );
        assert_eq!(RelayLink::normalize("robot"), "rerun+http://robot/proxy");
        assert_eq!(
            RelayLink::normalize("rerun+http://10.0.0.5:9876"),
            "rerun+http://10.0.0.5:9876/proxy"
        );
        assert_eq!(
            RelayLink::normalize("rerun+http://10.0.0.5:9876/proxy"),
            "rerun+http://10.0.0.5:9876/proxy"
        );
    }

    #[test]
    fn forward_marks_connected_and_forwards_messages() {
        let (inner_tx, inner_rx) = channel();
        let (tx, rx) = channel();
        let state = Mutex::new(ConnectionState::Connecting);

        inner_tx.send(message()).unwrap();
        inner_tx.quit(None).unwrap();

        let outcome = RelayLink::forward(&inner_rx, &tx, &state);
        assert!(matches!(outcome, Forward::StreamEnded { delivered: true }));
        assert_eq!(*state.lock().unwrap(), ConnectionState::Connected);
        assert!(rx.recv().unwrap().data().is_some());
    }

    #[test]
    fn forward_stays_connecting_on_a_dead_dial() {
        let (inner_tx, inner_rx) = channel();
        let (tx, _rx) = channel();
        let state = Mutex::new(ConnectionState::Connecting);

        inner_tx
            .quit(Some(Box::new(std::io::Error::other("connection refused"))))
            .unwrap();

        let outcome = RelayLink::forward(&inner_rx, &tx, &state);
        assert!(matches!(outcome, Forward::StreamEnded { delivered: false }));
        assert_eq!(*state.lock().unwrap(), ConnectionState::Connecting);
    }

    #[test]
    fn forward_stops_when_the_viewer_receiver_closes() {
        let (inner_tx, inner_rx) = channel();
        let (tx, rx) = channel();
        let state = Mutex::new(ConnectionState::Connecting);

        drop(rx);
        inner_tx.send(message()).unwrap();

        let outcome = RelayLink::forward(&inner_rx, &tx, &state);
        assert!(matches!(outcome, Forward::ReceiverClosed));
    }
}
