use std::cmp::Ordering;

use re_sdk_types::ComponentDescriptor;
use re_ui::Icon;
use rewire_extras::ROS2NodeInfo;

use super::table::{Columns, TableSpec};

/// Table of discovered ROS 2 nodes, read from [`ROS2NodeInfo`] at `/rewire/nodes`.
pub struct Nodes;

/// One node.
pub struct NodeRow {
    node_name: String,
    publishers: usize,
    subscribers: usize,
    transport: String,
}

impl TableSpec for Nodes {
    type Row = NodeRow;
    const NAME: &'static str = "Nodes";
    const ICON: &'static Icon = &Icon::new(
        "view_nodes.svg",
        include_bytes!("../../assets/icons/view_nodes.svg"),
    );
    const ENTITY_PATH: &'static str = "/rewire/nodes";
    const EMPTY: &'static str = "No nodes yet";
    const COLUMNS: &'static [(&'static str, f32)] = &[
        ("Node", 120.0),
        ("Pubs", 30.0),
        ("Subs", 30.0),
        ("Transport", 50.0),
    ];

    fn descriptors() -> Vec<ComponentDescriptor> {
        vec![
            ROS2NodeInfo::descriptor_node_name(),
            ROS2NodeInfo::descriptor_publisher_count(),
            ROS2NodeInfo::descriptor_subscriber_count(),
            ROS2NodeInfo::descriptor_transport(),
        ]
    }

    fn rows(cols: &Columns) -> Vec<NodeRow> {
        (0..cols.row_count())
            .map(|i| NodeRow {
                node_name: cols.text(0, i),
                publishers: cols.parse(1, i).unwrap_or(0),
                subscribers: cols.parse(2, i).unwrap_or(0),
                transport: cols.text(3, i),
            })
            .collect()
    }

    fn cmp(a: &NodeRow, b: &NodeRow, col: usize) -> Ordering {
        match col {
            0 => a.node_name.cmp(&b.node_name),
            1 => a.publishers.cmp(&b.publishers),
            2 => a.subscribers.cmp(&b.subscribers),
            _ => a.transport.cmp(&b.transport),
        }
    }

    fn cell(ui: &mut egui::Ui, row: &NodeRow, col: usize) {
        match col {
            0 => ui.label(&row.node_name),
            1 => ui.label(row.publishers.to_string()),
            2 => ui.label(row.subscribers.to_string()),
            _ => ui.label(&row.transport),
        };
    }
}
