mod diagnostics;
mod nodes;
mod table;
mod topics;

/// SpaceView for per-topic diagnostics (Hz, throughput, drops, latency).
pub type DiagnosticsView = table::TableView<diagnostics::Diagnostics>;
/// SpaceView listing discovered ROS 2 nodes.
pub type NodesView = table::TableView<nodes::Nodes>;
/// SpaceView listing discovered ROS 2 topics.
pub type TopicsView = table::TableView<topics::Topics>;
