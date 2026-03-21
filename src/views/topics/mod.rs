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

use self::system::TopicsSystem;

#[derive(Default)]
pub struct TopicsView;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Topic,
    Type,
    Pubs,
    Subs,
}

struct TopicsViewState {
    sort_column: SortColumn,
    ascending: bool,
}

impl Default for TopicsViewState {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Topic,
            ascending: true,
        }
    }
}

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
        state: &mut dyn ViewState,
        _query: &ViewQuery<'_>,
        system_output: SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let tokens = ui.tokens();
        let state = state.downcast_mut::<TopicsViewState>()?;
        let topics = system_output.view_systems.get::<TopicsSystem>()?;

        if topics.entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.weak("No topics yet");
            });
            return Ok(());
        }

        let mut sorted: Vec<&system::TopicEntry> = topics.entries.iter().collect();
        match state.sort_column {
            SortColumn::Topic => sorted.sort_by(|a, b| a.topic_name.cmp(&b.topic_name)),
            SortColumn::Type => sorted.sort_by(|a, b| a.type_name.cmp(&b.type_name)),
            SortColumn::Pubs => sorted.sort_by(|a, b| a.publishers.cmp(&b.publishers)),
            SortColumn::Subs => sorted.sort_by(|a, b| a.subscribers.cmp(&b.subscribers)),
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
                .column(Column::auto().at_least(100.0).clip(true))
                .column(Column::auto().at_least(120.0).clip(true))
                .column(Column::auto().at_least(30.0))
                .column(Column::remainder().at_least(30.0))
                .header(tokens.deprecated_table_header_height(), |mut header| {
                    re_ui::DesignTokens::setup_table_header(&mut header);
                    for (label, col) in [
                        ("Topic", SortColumn::Topic),
                        ("Type", SortColumn::Type),
                        ("Pubs", SortColumn::Pubs),
                        ("Subs", SortColumn::Subs),
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
                            ui.label(&entry.topic_name);
                        });
                        row.col(|ui| {
                            ui.label(&entry.type_name);
                        });
                        row.col(|ui| {
                            ui.label(entry.publishers.to_string());
                        });
                        row.col(|ui| {
                            ui.label(entry.subscribers.to_string());
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
