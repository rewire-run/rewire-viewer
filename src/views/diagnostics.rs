use std::cmp::Ordering;

use re_sdk_types::ComponentDescriptor;
use re_ui::Icon;
use rewire_extras::ROS2DiagnosticsInfo;

use super::table::{Columns, TableSpec};

/// Per-topic diagnostics (Hz, throughput, drops, latency), read from
/// [`ROS2DiagnosticsInfo`] at `/rewire/diagnostics`.
pub struct Diagnostics;

/// One topic's diagnostics.
pub struct DiagnosticsRow {
    topic: String,
    hz: f64,
    bytes_per_sec: f64,
    drops: u64,
    latency_ms: Option<f64>,
    /// Bridge-side rate cap, if throttled; `hz` stays the wire rate.
    max_hz: Option<f64>,
}

impl TableSpec for Diagnostics {
    type Row = DiagnosticsRow;
    const NAME: &'static str = "Diagnostics";
    const ICON: &'static Icon = &Icon::new(
        "view_diagnostics.svg",
        include_bytes!("../../assets/icons/view_diagnostics.svg"),
    );
    const ENTITY_PATH: &'static str = "/rewire/diagnostics";
    const EMPTY: &'static str = "No diagnostics yet — enable with --diagnostics";
    const COLUMNS: &'static [(&'static str, f32)] = &[
        ("Topic", 120.0),
        ("Hz", 50.0),
        ("Bytes/s", 70.0),
        ("Drops", 40.0),
        ("Latency", 60.0),
    ];

    fn descriptors() -> Vec<ComponentDescriptor> {
        vec![
            ROS2DiagnosticsInfo::descriptor_topic_name(),
            ROS2DiagnosticsInfo::descriptor_hz(),
            ROS2DiagnosticsInfo::descriptor_bytes_per_sec(),
            ROS2DiagnosticsInfo::descriptor_drops(),
            ROS2DiagnosticsInfo::descriptor_latency_ms(),
            ROS2DiagnosticsInfo::descriptor_max_hz(),
        ]
    }

    fn rows(cols: &Columns) -> Vec<DiagnosticsRow> {
        (0..cols.row_count())
            .map(|i| DiagnosticsRow {
                topic: cols.text(0, i),
                hz: cols.parse(1, i).unwrap_or(0.0),
                bytes_per_sec: cols.parse(2, i).unwrap_or(0.0),
                drops: cols.parse(3, i).unwrap_or(0),
                latency_ms: cols.parse(4, i),
                max_hz: cols.parse(5, i),
            })
            .collect()
    }

    fn cmp(a: &DiagnosticsRow, b: &DiagnosticsRow, col: usize) -> Ordering {
        match col {
            0 => a.topic.cmp(&b.topic),
            1 => a.hz.total_cmp(&b.hz),
            2 => a.bytes_per_sec.total_cmp(&b.bytes_per_sec),
            3 => a.drops.cmp(&b.drops),
            _ => a
                .latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(Ordering::Equal),
        }
    }

    fn cell(ui: &mut egui::Ui, row: &DiagnosticsRow, col: usize) {
        match col {
            0 => {
                ui.label(&row.topic);
            }
            1 => {
                ui.label(format!("{:.1}", row.hz));
                if let Some(cap) = row.max_hz {
                    ui.weak(format!("(capped {cap:.1})"));
                }
            }
            2 => {
                ui.label(format_bytes_per_sec(row.bytes_per_sec));
            }
            3 => {
                ui.label(row.drops.to_string());
            }
            _ => {
                ui.label(
                    row.latency_ms
                        .map_or_else(|| "—".to_owned(), |ms| format!("{ms:.1} ms")),
                );
            }
        }
    }
}

fn format_bytes_per_sec(bps: f64) -> String {
    if bps >= 1_000_000.0 {
        format!("{:.1} MB/s", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.1} KB/s", bps / 1_000.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}
