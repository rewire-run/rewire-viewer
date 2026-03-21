use std::time::Duration;

use rerun::external::{arrow, egui, re_chunk_store, re_entity_db, re_log_types};
use rewire_extras::{ROS2NodeInfo, ROS2TopicInfo};

pub struct StatusBar {
    has_db: bool,
    connected: bool,
    bridge_count: usize,
    node_count: usize,
    topic_count: usize,
    app_id: String,
    uptime: Duration,
}

impl StatusBar {
    pub fn new(
        db: Option<&re_entity_db::EntityDb>,
        connected: bool,
        bridge_count: usize,
        uptime: Duration,
    ) -> Self {
        Self {
            has_db: db.is_some(),
            connected,
            bridge_count,
            node_count: db.map(node_count).unwrap_or(0),
            topic_count: db.map(topic_count).unwrap_or(0),
            app_id: db
                .and_then(|db| db.store_info().map(|i| i.application_id().to_string()))
                .unwrap_or_default(),
            uptime,
        }
    }

    pub fn render(&self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.add_space(8.0);

            if !self.has_db {
                ui.colored_label(egui::Color32::GRAY, "⬤");
                ui.label("Waiting for connection...");
                return;
            }

            if self.connected {
                ui.colored_label(egui::Color32::from_rgb(80, 200, 120), "⬤");
                ui.label("Connected");
            } else {
                ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "⬤");
                ui.label("Disconnected");
            }

            ui.separator();

            let suffix = if self.bridge_count == 1 { "" } else { "s" };
            ui.label(format!("{} bridge{suffix}", self.bridge_count));
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

fn node_count(entity_db: &re_entity_db::EntityDb) -> usize {
    let timeline = re_log_types::TimelineName::log_time();
    let query = re_chunk_store::LatestAtQuery::latest(timeline);
    let path = re_log_types::EntityPath::from("/rewire/nodes");
    let id = ROS2NodeInfo::descriptor_node_name().component;

    entity_db
        .storage_engine()
        .cache()
        .latest_at(&query, &path, [id])
        .component_batch_raw(id)
        .map(|arr| {
            use arrow::array::Array as _;
            arr.len()
        })
        .unwrap_or(0)
}

fn topic_count(entity_db: &re_entity_db::EntityDb) -> usize {
    let timeline = re_log_types::TimelineName::log_time();
    let query = re_chunk_store::LatestAtQuery::latest(timeline);
    let path = re_log_types::EntityPath::from("/rewire/topics");
    let id = ROS2TopicInfo::descriptor_topic_name().component;

    entity_db
        .storage_engine()
        .cache()
        .latest_at(&query, &path, [id])
        .component_batch_raw(id)
        .map(|arr| {
            use arrow::array::Array as _;
            arr.len()
        })
        .unwrap_or(0)
}
