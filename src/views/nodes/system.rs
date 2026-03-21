use rerun::external::{re_chunk_store, re_log_types, re_viewer_context};

use re_chunk_store::LatestAtQuery;
use re_log_types::{EntityPath, TimelineName};
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewQuery, ViewSystemExecutionError, ViewSystemIdentifier,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};

use rewire_extras::ROS2NodeInfo;

pub struct NodeEntry {
    pub node_name: String,
    pub publishers: usize,
    pub subscribers: usize,
    pub transport: String,
}

#[derive(Default)]
pub struct NodesSystem {
    pub entries: Vec<NodeEntry>,
}

impl IdentifiedViewSystem for NodesSystem {
    fn identifier() -> ViewSystemIdentifier {
        "Nodes".into()
    }
}

impl VisualizerSystem for NodesSystem {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        let mut info = VisualizerQueryInfo::empty();
        info.queried.insert(ROS2NodeInfo::descriptor_node_name());
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

        let node_name_id = ROS2NodeInfo::descriptor_node_name().component;
        let pub_count_id = ROS2NodeInfo::descriptor_publisher_count().component;
        let sub_count_id = ROS2NodeInfo::descriptor_subscriber_count().component;
        let transport_id = ROS2NodeInfo::descriptor_transport().component;

        let entity_path = EntityPath::from("/rewire/nodes");

        self.entries.clear();

        let results = entity_db.storage_engine().cache().latest_at(
            &query,
            &entity_path,
            [node_name_id, pub_count_id, sub_count_id, transport_id],
        );

        let names = results
            .component_batch_raw(node_name_id)
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
        let transports = results
            .component_batch_raw(transport_id)
            .map(|arr| crate::util::extract_texts(&arr))
            .unwrap_or_default();

        for i in 0..names.len() {
            self.entries.push(NodeEntry {
                node_name: names.get(i).cloned().unwrap_or_default(),
                publishers: pub_counts.get(i).and_then(|s| s.parse().ok()).unwrap_or(0),
                subscribers: sub_counts.get(i).and_then(|s| s.parse().ok()).unwrap_or(0),
                transport: transports.get(i).cloned().unwrap_or_default(),
            });
        }

        Ok(VisualizerExecutionOutput::default())
    }

    fn data(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
