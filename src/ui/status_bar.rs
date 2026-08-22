use std::time::Duration;

use rewire_extras::{ROS2NodeInfo, ROS2TopicInfo};

use crate::connection::{ConnectionState, FleetSnapshot};

/// Bottom bar showing connection state, bridge fleet, node/topic counts, and uptime.
pub struct StatusBar {
    has_db: bool,
    state: ConnectionState,
    fleet: Option<FleetSnapshot>,
    node_count: usize,
    topic_count: usize,
    app_id: String,
    uptime: Duration,
}

impl StatusBar {
    /// Snapshots the current viewer state for rendering.
    pub fn new(
        db: Option<&re_entity_db::EntityDb>,
        state: ConnectionState,
        fleet: Option<FleetSnapshot>,
        uptime: Duration,
    ) -> Self {
        Self {
            has_db: db.is_some(),
            state,
            fleet,
            node_count: db.map_or(0, |db| {
                latest_len(
                    db,
                    "/rewire/nodes",
                    ROS2NodeInfo::descriptor_node_name().component,
                )
            }),
            topic_count: db.map_or(0, |db| {
                latest_len(
                    db,
                    "/rewire/topics",
                    ROS2TopicInfo::descriptor_topic_name().component,
                )
            }),
            app_id: db
                .and_then(|db| db.store_info().map(|i| i.application_id().to_string()))
                .unwrap_or_default(),
            uptime,
        }
    }

    /// Draws the status bar into the given `Ui`.
    pub fn render(&self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.add_space(8.0);

            match self.state {
                ConnectionState::Connecting => {
                    ui.colored_label(egui::Color32::GRAY, "⬤");
                    ui.label("Connecting...");
                }
                ConnectionState::Connected => {
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 120), "⬤");
                    ui.label("Connected");
                }
                ConnectionState::Reconnecting => {
                    ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "⬤");
                    ui.label("Reconnecting...");
                }
            }

            if let Some(fleet) = &self.fleet {
                ui.separator();
                let suffix = if fleet.bridge_count == 1 { "" } else { "s" };
                let activity = if fleet.bridge_count == 0 {
                    ""
                } else if fleet.any_active {
                    " (active)"
                } else {
                    " (idle)"
                };
                ui.label(format!("{} bridge{suffix}{activity}", fleet.bridge_count));
            }

            if !self.has_db {
                return;
            }

            ui.separator();

            if !self.app_id.is_empty() {
                ui.label(format!("App: {}", self.app_id));
                ui.separator();
            }

            let node_suffix = if self.node_count == 1 { "" } else { "s" };
            ui.label(format!("{} node{node_suffix}", self.node_count));
            ui.separator();

            ui.label(format!("{} topics", self.topic_count));
            ui.separator();

            let secs = self.uptime.as_secs();
            let mins = secs / 60;
            let hours = mins / 60;
            if hours > 0 {
                ui.label(format!("{}h {}m", hours, mins % 60));
            } else if mins > 0 {
                ui.label(format!("{}m {}s", mins, secs % 60));
            } else {
                ui.label(format!("{}s", secs));
            }
        });
    }
}

fn latest_len(
    db: &re_entity_db::EntityDb,
    path: &str,
    id: re_sdk_types::ComponentIdentifier,
) -> usize {
    let query = re_chunk_store::LatestAtQuery::latest(re_log_types::TimelineName::log_time());
    db.storage_engine()
        .cache()
        .latest_at(
            re_chunk_store::ChunkTrackingMode::Ignore,
            &query,
            &re_log_types::EntityPath::from(path),
            [id],
        )
        .component_batch_raw(id)
        .map_or(0, |arr| arrow::array::Array::len(&*arr))
}
