use rerun::external::{re_log_types, re_sdk_types, re_viewer_context};

use re_log_types::EntityPath;
use re_sdk_types::ComponentIdentifier;
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewQuery, ViewSystemIdentifier, VisualizerExecutionOutput,
    VisualizerQueryInfo, VisualizerSystem, ViewSystemExecutionError,
};

pub struct TopicEntry {
    pub entity_path: EntityPath,
    pub components: Vec<String>,
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
        let timeline = re_log_types::TimelineName::log_time();
        let entity_db = ctx.viewer_ctx.recording();

        self.entries.clear();
        let paths: Vec<EntityPath> = entity_db
            .sorted_entity_paths()
            .cloned()
            .collect::<Vec<EntityPath>>();

        for entity_path in &paths {
            let path_str = format!("{entity_path}");
            if path_str.starts_with("/rewire") || path_str.starts_with("/bridge") {
                continue;
            }

            let components: Vec<String> = entity_db
                .storage_engine()
                .store()
                .all_components_on_timeline_sorted(&timeline, entity_path)
                .map(|set: std::collections::BTreeSet<ComponentIdentifier>| {
                    set.iter().map(|ci| ci.to_string()).collect()
                })
                .unwrap_or_default();

            if !components.is_empty() {
                self.entries.push(TopicEntry {
                    entity_path: entity_path.clone(),
                    components,
                });
            }
        }

        Ok(VisualizerExecutionOutput::default())
    }

    fn data(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
