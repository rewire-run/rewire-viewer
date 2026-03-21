pub mod system;

use re_ui::UiExt as _;
use egui;
use re_chunk_store;
use re_log_types;
use re_sdk_types;
use re_ui;
use re_viewer_context;

use re_log_types::EntityPath;
use re_sdk_types::ViewClassIdentifier;
use re_viewer_context::{
    SystemExecutionOutput, ViewClass, ViewClassLayoutPriority, ViewClassRegistryError, ViewQuery,
    ViewSpawnHeuristics, ViewState, ViewStateExt as _, ViewSystemExecutionError,
    ViewSystemRegistrator, ViewerContext,
};

use self::system::DiagnosticsSystem;

#[derive(Default)]
pub struct DiagnosticsView;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Topic,
    Hz,
    BytesPerSec,
    Drops,
    Latency,
}

struct DiagnosticsViewState {
    sort_column: SortColumn,
    ascending: bool,
}

impl Default for DiagnosticsViewState {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Topic,
            ascending: true,
        }
    }
}

impl ViewState for DiagnosticsViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ViewClass for DiagnosticsView {
    fn identifier() -> ViewClassIdentifier
    where
        Self: Sized,
    {
        "Diagnostics".into()
    }

    fn display_name(&self) -> &'static str {
        "Diagnostics"
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &re_ui::icons::VIEW_TEXT
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> re_ui::Help {
        re_ui::Help::new("Diagnostics View")
    }

    fn on_register(
        &self,
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<DiagnosticsSystem>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<DiagnosticsViewState>::default()
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
        let state = state.downcast_mut::<DiagnosticsViewState>()?;
        let diag = system_output.view_systems.get::<DiagnosticsSystem>()?;

        if diag.entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.weak("No diagnostics yet — enable with --diagnostics");
            });
            return Ok(());
        }

        let mut sorted: Vec<&system::DiagnosticsEntry> = diag.entries.iter().collect();
        match state.sort_column {
            SortColumn::Topic => sorted.sort_by(|a, b| a.topic.cmp(&b.topic)),
            SortColumn::Hz => {
                sorted.sort_by(|a, b| a.hz.partial_cmp(&b.hz).unwrap_or(std::cmp::Ordering::Equal))
            }
            SortColumn::BytesPerSec => sorted.sort_by(|a, b| {
                a.bytes_per_sec
                    .partial_cmp(&b.bytes_per_sec)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortColumn::Drops => sorted.sort_by(|a, b| a.drops.cmp(&b.drops)),
            SortColumn::Latency => sorted.sort_by(|a, b| {
                a.latency_ms
                    .partial_cmp(&b.latency_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
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
                .column(Column::auto().at_least(50.0))
                .column(Column::auto().at_least(70.0))
                .column(Column::auto().at_least(40.0))
                .column(Column::remainder().at_least(60.0))
                .header(tokens.deprecated_table_header_height(), |mut header| {
                    re_ui::DesignTokens::setup_table_header(&mut header);
                    for (label, col) in [
                        ("Topic", SortColumn::Topic),
                        ("Hz", SortColumn::Hz),
                        ("Bytes/s", SortColumn::BytesPerSec),
                        ("Drops", SortColumn::Drops),
                        ("Latency", SortColumn::Latency),
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
                            ui.label(&entry.topic);
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.1}", entry.hz));
                        });
                        row.col(|ui| {
                            ui.label(format_bytes_per_sec(entry.bytes_per_sec));
                        });
                        row.col(|ui| {
                            ui.label(entry.drops.to_string());
                        });
                        row.col(|ui| {
                            ui.label(match entry.latency_ms {
                                Some(ms) => format!("{ms:.1} ms"),
                                None => "—".to_owned(),
                            });
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

fn format_bytes_per_sec(bps: f64) -> String {
    if bps >= 1_000_000.0 {
        format!("{:.1} MB/s", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.1} KB/s", bps / 1_000.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}
