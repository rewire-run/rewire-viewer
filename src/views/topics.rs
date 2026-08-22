use std::cmp::Ordering;

use re_sdk_types::ComponentDescriptor;
use re_ui::Icon;
use rewire_extras::ROS2TopicInfo;

use super::table::{Columns, TableSpec};

/// Table of discovered ROS 2 topics, read from [`ROS2TopicInfo`] at `/rewire/topics`.
pub struct Topics;

/// One topic.
pub struct TopicRow {
    topic_name: String,
    type_name: String,
    publishers: usize,
    subscribers: usize,
}

impl TableSpec for Topics {
    type Row = TopicRow;
    const NAME: &'static str = "Topics";
    const ICON: &'static Icon = &Icon::new(
        "view_topics.svg",
        include_bytes!("../../assets/icons/view_topics.svg"),
    );
    const ENTITY_PATH: &'static str = "/rewire/topics";
    const EMPTY: &'static str = "No topics yet";
    const COLUMNS: &'static [(&'static str, f32)] = &[
        ("Topic", 100.0),
        ("Type", 120.0),
        ("Pubs", 30.0),
        ("Subs", 30.0),
    ];

    fn descriptors() -> Vec<ComponentDescriptor> {
        vec![
            ROS2TopicInfo::descriptor_topic_name(),
            ROS2TopicInfo::descriptor_type_name(),
            ROS2TopicInfo::descriptor_publisher_count(),
            ROS2TopicInfo::descriptor_subscriber_count(),
        ]
    }

    fn rows(cols: &Columns) -> Vec<TopicRow> {
        (0..cols.row_count())
            .map(|i| TopicRow {
                topic_name: cols.text(0, i),
                type_name: cols.text(1, i),
                publishers: cols.parse(2, i).unwrap_or(0),
                subscribers: cols.parse(3, i).unwrap_or(0),
            })
            .collect()
    }

    fn cmp(a: &TopicRow, b: &TopicRow, col: usize) -> Ordering {
        match col {
            0 => a.topic_name.cmp(&b.topic_name),
            1 => a.type_name.cmp(&b.type_name),
            2 => a.publishers.cmp(&b.publishers),
            _ => a.subscribers.cmp(&b.subscribers),
        }
    }

    fn cell(ui: &mut egui::Ui, row: &TopicRow, col: usize) {
        match col {
            0 => ui.label(&row.topic_name),
            1 => ui.label(&row.type_name),
            2 => ui.label(row.publishers.to_string()),
            _ => ui.label(row.subscribers.to_string()),
        };
    }
}
