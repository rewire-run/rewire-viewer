pub mod system;

use re_ui::UiExt as _;
use rerun::external::{egui, re_chunk_store, re_log_types, re_sdk_types, re_ui, re_viewer_context};

use re_log_types::EntityPath;
use re_sdk_types::ViewClassIdentifier;
use re_viewer_context::{
    SystemExecutionOutput, ViewClass, ViewClassLayoutPriority, ViewClassRegistryError, ViewQuery,
    ViewSpawnHeuristics, ViewState, ViewStateExt as _, ViewSystemExecutionError,
    ViewSystemRegistrator, ViewerContext,
};

use self::system::NodesSystem;

#[derive(Default)]
pub struct NodesView;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Node,
    Pubs,
    Subs,
    Transport,
}

struct NodesViewState {
    sort_column: SortColumn,
    ascending: bool,
}

impl Default for NodesViewState {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Node,
            ascending: true,
        }
    }
}

impl ViewState for NodesViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ViewClass for NodesView {
    fn identifier() -> ViewClassIdentifier
    where
        Self: Sized,
    {
        "Nodes".into()
    }

    fn display_name(&self) -> &'static str {
        "Nodes"
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &re_ui::icons::VIEW_TEXT
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> re_ui::Help {
        re_ui::Help::new("Nodes View")
    }

    fn on_register(
        &self,
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<NodesSystem>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<NodesViewState>::default()
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
        state: &mut dyn ViewState,
        _query: &ViewQuery<'_>,
        system_output: SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let tokens = ui.tokens();
        let state = state.downcast_mut::<NodesViewState>()?;
        let nodes = system_output.view_systems.get::<NodesSystem>()?;

        if nodes.entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.weak("No nodes yet");
            });
            return Ok(());
        }

        let mut sorted: Vec<&system::NodeEntry> = nodes.entries.iter().collect();
        match state.sort_column {
            SortColumn::Node => sorted.sort_by(|a, b| a.node_name.cmp(&b.node_name)),
            SortColumn::Pubs => sorted.sort_by(|a, b| a.publishers.cmp(&b.publishers)),
            SortColumn::Subs => sorted.sort_by(|a, b| a.subscribers.cmp(&b.subscribers)),
            SortColumn::Transport => sorted.sort_by(|a, b| a.transport.cmp(&b.transport)),
        }
        if !state.ascending {
            sorted.reverse();
        }

        use egui_extras::Column;

        let table_style = re_ui::TableStyle::Dense;
        let row_height = tokens.table_row_height(table_style);
        let sort_col = state.sort_column;
        let sort_asc = state.ascending;

        let mut clicked_col: Option<SortColumn> = None;

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
                .column(Column::auto().at_least(120.0).clip(true))
                .column(Column::auto().at_least(30.0))
                .column(Column::auto().at_least(30.0))
                .column(Column::remainder().at_least(50.0))
                .header(tokens.deprecated_table_header_height(), |mut header| {
                    re_ui::DesignTokens::setup_table_header(&mut header);
                    for (label, col) in [
                        ("Node", SortColumn::Node),
                        ("Pubs", SortColumn::Pubs),
                        ("Subs", SortColumn::Subs),
                        ("Transport", SortColumn::Transport),
                    ] {
                        header.col(|ui| {
                            crate::ui::sortable_header(
                                ui,
                                label,
                                sort_col == col,
                                sort_asc,
                                &mut clicked_col,
                                col,
                            );
                        });
                    }
                })
                .body(|mut body| {
                    tokens.setup_table_body(&mut body, table_style);
                    body.rows(row_height, sorted.len(), |mut row| {
                        let entry = sorted[row.index()];
                        row.col(|ui| {
                            ui.label(&entry.node_name);
                        });
                        row.col(|ui| {
                            ui.label(entry.publishers.to_string());
                        });
                        row.col(|ui| {
                            ui.label(entry.subscribers.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&entry.transport);
                        });
                    });
                });
        });

        if let Some(col) = clicked_col {
            if state.sort_column == col {
                state.ascending = !state.ascending;
            } else {
                state.sort_column = col;
                state.ascending = true;
            }
        }

        Ok(())
    }
}
