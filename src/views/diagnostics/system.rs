use rerun::external::{re_chunk_store, re_log_types, re_viewer_context};

use re_chunk_store::LatestAtQuery;
use re_log_types::{EntityPath, TimelineName};
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewQuery, ViewSystemExecutionError, ViewSystemIdentifier,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};

use rewire_extras::ROS2DiagnosticsInfo;

pub struct DiagnosticsEntry {
    pub topic: String,
    pub hz: f64,
    pub bytes_per_sec: f64,
    pub drops: u64,
    pub latency_ms: Option<f64>,
}

#[derive(Default)]
pub struct DiagnosticsSystem {
    pub entries: Vec<DiagnosticsEntry>,
}

impl IdentifiedViewSystem for DiagnosticsSystem {
    fn identifier() -> ViewSystemIdentifier {
        "Diagnostics".into()
    }
}

impl VisualizerSystem for DiagnosticsSystem {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        let mut info = VisualizerQueryInfo::empty();
        info.queried
            .insert(ROS2DiagnosticsInfo::descriptor_topic_name());
        info
    }

    fn execute(
        &mut self,
        ctx: &ViewContext<'_>,
        _query: &ViewQuery<'_>,
        _context_systems: &re_viewer_context::ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let entity_db = ctx.viewer_ctx.recording();
        let timeline = TimelineName::log_time();
        let query = LatestAtQuery::latest(timeline);

        let topic_id = ROS2DiagnosticsInfo::descriptor_topic_name().component;
        let hz_id = ROS2DiagnosticsInfo::descriptor_hz().component;
        let bps_id = ROS2DiagnosticsInfo::descriptor_bytes_per_sec().component;
        let drops_id = ROS2DiagnosticsInfo::descriptor_drops().component;
        let latency_id = ROS2DiagnosticsInfo::descriptor_latency_ms().component;

        let entity_path = EntityPath::from("/rewire/diagnostics");

        self.entries.clear();

        let results = entity_db.storage_engine().cache().latest_at(
            &query,
            &entity_path,
            [topic_id, hz_id, bps_id, drops_id, latency_id],
        );

        let topics = results
            .component_batch_raw(topic_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let hz_vals = results
            .component_batch_raw(hz_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let bps_vals = results
            .component_batch_raw(bps_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let drop_vals = results
            .component_batch_raw(drops_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let latency_vals = results
            .component_batch_raw(latency_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();

        for i in 0..topics.len() {
            let latency_str = latency_vals.get(i).map(|s| s.as_str()).unwrap_or("");
            self.entries.push(DiagnosticsEntry {
                topic: topics.get(i).cloned().unwrap_or_default(),
                hz: hz_vals.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                bytes_per_sec: bps_vals.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                drops: drop_vals.get(i).and_then(|s| s.parse().ok()).unwrap_or(0),
                latency_ms: if latency_str.is_empty() {
                    None
                } else {
                    latency_str.parse().ok()
                },
            });
        }

        Ok(VisualizerExecutionOutput::default())
    }

    fn data(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
