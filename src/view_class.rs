use rerun::external::{
    egui, re_chunk_store, re_log_types, re_sdk_types, re_ui, re_viewer_context,
};

use re_log_types::EntityPath;
use re_sdk_types::ViewClassIdentifier;
use re_viewer_context::{
    SystemExecutionOutput, ViewClass, ViewClassLayoutPriority, ViewClassRegistryError, ViewQuery,
    ViewSpawnHeuristics, ViewState, ViewSystemExecutionError, ViewSystemRegistrator, ViewerContext,
};

use crate::topics_system::TopicsSystem;

#[derive(Default)]
pub struct TopicsView;

#[derive(Default)]
struct TopicsViewState;

impl ViewState for TopicsViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ViewClass for TopicsView {
    fn identifier() -> ViewClassIdentifier
    where
        Self: Sized,
    {
        "Topics".into()
    }

    fn display_name(&self) -> &'static str {
        "Topics"
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &re_ui::icons::VIEW_TEXT
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> re_ui::Help {
        re_ui::Help::new("Topics View")
    }

    fn on_register(
        &self,
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<TopicsSystem>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<TopicsViewState>::default()
    }

    fn layout_priority(&self) -> ViewClassLayoutPriority {
        ViewClassLayoutPriority::Low
    }

    fn spawn_heuristics(
        &self,
        _ctx: &ViewerContext<'_>,
        _include_entity: &dyn Fn(&EntityPath) -> bool,
    ) -> ViewSpawnHeuristics {
        ViewSpawnHeuristics::empty()
    }

    fn ui(
        &self,
        _ctx: &ViewerContext<'_>,
        _missing_chunk_reporter: &re_chunk_store::MissingChunkReporter,
        ui: &mut egui::Ui,
        _state: &mut dyn ViewState,
        _query: &ViewQuery<'_>,
        system_output: SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let topics = system_output.view_systems.get::<TopicsSystem>()?;

        if topics.entries.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No topics yet");
            });
            return Ok(());
        }

        use egui_extras::{Column, TableBuilder};

        TableBuilder::new(ui)
            .resizable(true)
            .vscroll(true)
            .column(Column::auto().at_least(200.0))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Topic");
                });
                header.col(|ui| {
                    ui.strong("Archetype");
                });
            })
            .body(|body| {
                body.rows(20.0, topics.entries.len(), |mut row| {
                    let entry = &topics.entries[row.index()];
                    row.col(|ui| {
                        ui.label(entry.entity_path.to_string());
                    });
                    row.col(|ui| {
                        ui.label(entry.components.join(", "));
                    });
                });
            });

        Ok(())
    }
}
