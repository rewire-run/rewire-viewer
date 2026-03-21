use rerun::external::{re_chunk_store, re_log_types, re_viewer_context};

use re_chunk_store::LatestAtQuery;
use re_log_types::{EntityPath, TimelineName};
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewQuery, ViewSystemIdentifier, VisualizerExecutionOutput,
    VisualizerQueryInfo, VisualizerSystem, ViewSystemExecutionError,
};

use rewire_extras::ROS2TopicInfo;

pub struct TopicEntry {
    pub topic_name: String,
    pub type_name: String,
    pub publishers: usize,
    pub subscribers: usize,
}

#[derive(Default)]
pub struct TopicsSystem {
    pub entries: Vec<TopicEntry>,
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
        let mut info = VisualizerQueryInfo::empty();
        info.queried.insert(ROS2TopicInfo::descriptor_topic_name());
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

        let topic_name_id = ROS2TopicInfo::descriptor_topic_name().component;
        let type_name_id = ROS2TopicInfo::descriptor_type_name().component;
        let pub_count_id = ROS2TopicInfo::descriptor_publisher_count().component;
        let sub_count_id = ROS2TopicInfo::descriptor_subscriber_count().component;

        let entity_path = EntityPath::from("/rewire/topics");

        self.entries.clear();

        let results = entity_db
            .storage_engine()
            .cache()
            .latest_at(&query, &entity_path, [topic_name_id, type_name_id, pub_count_id, sub_count_id]);

        let names = results
            .component_batch_raw(topic_name_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let types = results
            .component_batch_raw(type_name_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let pub_counts = results
            .component_batch_raw(pub_count_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();
        let sub_counts = results
            .component_batch_raw(sub_count_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();

        for i in 0..names.len() {
            self.entries.push(TopicEntry {
                topic_name: names.get(i).cloned().unwrap_or_default(),
                type_name: types.get(i).cloned().unwrap_or_default(),
                publishers: pub_counts.get(i).and_then(|s| s.parse().ok()).unwrap_or(0),
                subscribers: sub_counts.get(i).and_then(|s| s.parse().ok()).unwrap_or(0),
            });
        }

        Ok(VisualizerExecutionOutput::default())
    }

    fn data(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
