use rerun::external::{
    egui, re_chunk_store, re_log_types, re_sdk_types, re_ui, re_viewer_context,
};
use re_ui::UiExt as _;

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
        let tokens = ui.tokens();
        let topics = system_output.view_systems.get::<TopicsSystem>()?;

        if topics.entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.weak("No topics yet");
            });
            return Ok(());
        }

        use egui_extras::Column;

        let table_style = re_ui::TableStyle::Dense;
        let row_height = tokens.table_row_height(table_style);

        egui::Frame {
            inner_margin: tokens.view_padding().into(),
            ..egui::Frame::default()
        }
        .show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .resizable(true)
                .vscroll(true)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0)
                .max_scroll_height(f32::INFINITY)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().at_least(100.0).clip(true))
                .column(Column::auto().at_least(120.0).clip(true))
                .column(Column::auto().at_least(30.0))
                .column(Column::remainder().at_least(30.0))
                .header(tokens.deprecated_table_header_height(), |mut header| {
                    re_ui::DesignTokens::setup_table_header(&mut header);
                    header.col(|ui| { ui.strong("Topic"); });
                    header.col(|ui| { ui.strong("Type"); });
                    header.col(|ui| { ui.strong("Pubs"); });
                    header.col(|ui| { ui.strong("Subs"); });
                })
                .body(|mut body| {
                    tokens.setup_table_body(&mut body, table_style);
                    body.rows(row_height, topics.entries.len(), |mut row| {
                        let entry = &topics.entries[row.index()];
                        row.col(|ui| { ui.label(&entry.topic_name); });
                        row.col(|ui| { ui.label(&entry.type_name); });
                        row.col(|ui| { ui.label(entry.publishers.to_string()); });
                        row.col(|ui| { ui.label(entry.subscribers.to_string()); });
                    });
                });
        });

        Ok(())
    }
}
