use rerun::external::{
    re_chunk_store, re_log_types, re_sdk_types, re_viewer_context,
};

use rerun::external::arrow;

use re_chunk_store::LatestAtQuery;
use re_log_types::{EntityPath, TimelineName};
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewQuery, ViewSystemIdentifier, VisualizerExecutionOutput,
    VisualizerQueryInfo, VisualizerSystem, ViewSystemExecutionError,
};

use crate::archetype::ROS2TopicInfo;

pub struct TopicEntry {
    pub topic_name: String,
    pub type_name: String,
    pub status: String,
}

pub struct TopicsSystem {
    pub entries: Vec<TopicEntry>,
}

impl Default for TopicsSystem {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl IdentifiedViewSystem for TopicsSystem {
    fn identifier() -> ViewSystemIdentifier {
        "Topics".into()
    }
}

impl VisualizerSystem for TopicsSystem {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo::empty()
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

        let topic_name_desc = ROS2TopicInfo::descriptor_topic_name();
        let type_name_desc = ROS2TopicInfo::descriptor_type_name();
        let status_desc = ROS2TopicInfo::descriptor_status();

        let topic_name_id = topic_name_desc.component;
        let type_name_id = type_name_desc.component;
        let status_id = status_desc.component;

        self.entries.clear();

        let paths: Vec<EntityPath> = entity_db
            .sorted_entity_paths()
            .cloned()
            .collect::<Vec<EntityPath>>();

        for entity_path in &paths {
            let path_str = format!("{entity_path}");
            if !path_str.starts_with("/rewire/topics/") {
                continue;
            }

            let results = entity_db
                .storage_engine()
                .cache()
                .latest_at(&query, entity_path, [topic_name_id, type_name_id, status_id]);

            let topic_name = results
                .component_batch_raw(topic_name_id)
                .and_then(|arr| extract_text(&arr))
                .unwrap_or_else(|| path_str.trim_start_matches("/rewire/topics").to_string());

            let type_name = results
                .component_batch_raw(type_name_id)
                .and_then(|arr| extract_text(&arr))
                .unwrap_or_default();

            let status = results
                .component_batch_raw(status_id)
                .and_then(|arr| extract_text(&arr))
                .unwrap_or_else(|| "unknown".to_string());

            self.entries.push(TopicEntry {
                topic_name,
                type_name,
                status,
            });
        }

        Ok(VisualizerExecutionOutput::default())
    }

    fn data(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

fn extract_text(array: &dyn arrow::array::Array) -> Option<String> {
    use arrow::array::Array as _;
    let string_array = array
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()?;
    if string_array.is_empty() {
        return None;
    }
    Some(string_array.value(0).to_string())
}
