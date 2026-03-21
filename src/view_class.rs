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

        ui.add_space(4.0);

        if topics.entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("No topics yet")
                        .color(egui::Color32::from_gray(120)),
                );
            });
            return Ok(());
        }

        use egui_extras::{Column, TableBuilder};

        let row_height = 24.0;
        let header_height = 28.0;

        TableBuilder::new(ui)
            .resizable(true)
            .vscroll(true)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(160.0).clip(true))
            .column(Column::auto().at_least(180.0).clip(true))
            .column(Column::auto().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .header(header_height, |mut header| {
                header.col(|ui| {
                    ui.add_space(8.0);
                    ui.strong("Topic");
                });
                header.col(|ui| {
                    ui.add_space(8.0);
                    ui.strong("Type");
                });
                header.col(|ui| {
                    ui.add_space(8.0);
                    ui.strong("Pubs");
                });
                header.col(|ui| {
                    ui.add_space(8.0);
                    ui.strong("Subs");
                });
            })
            .body(|body| {
                body.rows(row_height, topics.entries.len(), |mut row| {
                    let entry = &topics.entries[row.index()];
                    row.col(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(&entry.topic_name).monospace(),
                        );
                    });
                    row.col(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(&entry.type_name)
                                .color(egui::Color32::from_gray(180)),
                        );
                    });
                    row.col(|ui| {
                        ui.add_space(8.0);
                        ui.label(entry.publishers.to_string());
                    });
                    row.col(|ui| {
                        ui.add_space(8.0);
                        ui.label(entry.subscribers.to_string());
                    });
                });
            });

        Ok(())
    }
}
