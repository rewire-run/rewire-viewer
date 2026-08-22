//! Generic sortable table view shared by the Topics, Nodes and Diagnostics panels.

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::str::FromStr;

use egui_extras::Column;
use re_chunk_store::LatestAtQuery;
use re_log_types::{EntityPath, TimelineName};
use re_sdk_types::{ComponentDescriptor, ViewClassIdentifier};
use re_ui::UiExt as _;
use re_viewer_context::{
    IdentifiedViewSystem, SystemExecutionOutput, ViewClass, ViewClassLayoutPriority,
    ViewClassRegistryError, ViewClassUiOutput, ViewContext, ViewQuery, ViewSpawnHeuristics,
    ViewState, ViewStateExt as _, ViewSystemExecutionError, ViewSystemIdentifier,
    ViewSystemRegistrator, ViewerContext, VisualizerExecutionOutput, VisualizerQueryInfo,
    VisualizerSystem,
};

/// Everything that distinguishes one table panel from another.
pub trait TableSpec: Send + Sync + 'static {
    /// One table row.
    type Row: Send + Sync + 'static;
    /// View identifier, display name and visualizer identifier.
    const NAME: &'static str;
    /// Icon shown in the view picker and tab.
    const ICON: &'static re_ui::Icon;
    /// Entity the bridge logs this table under.
    const ENTITY_PATH: &'static str;
    /// Placeholder shown while the table is empty.
    const EMPTY: &'static str;
    /// Column headers and minimum widths; the last column takes the remaining width.
    const COLUMNS: &'static [(&'static str, f32)];
    /// Components to fetch, in the order [`Columns`] indexes them.
    fn descriptors() -> Vec<ComponentDescriptor>;
    /// Builds rows from the fetched columns.
    fn rows(cols: &Columns) -> Vec<Self::Row>;
    /// Orders two rows by the given column index.
    fn cmp(a: &Self::Row, b: &Self::Row, col: usize) -> Ordering;
    /// Draws one cell.
    fn cell(ui: &mut egui::Ui, row: &Self::Row, col: usize);
}

/// Fetched component batches as text, one `Vec<String>` per descriptor.
pub struct Columns(Vec<Vec<String>>);

impl Columns {
    /// Number of rows, taken from the first column.
    pub fn row_count(&self) -> usize {
        self.0.first().map_or(0, Vec::len)
    }

    /// Cell text, or empty if the column is shorter than the table.
    pub fn text(&self, col: usize, row: usize) -> String {
        self.0[col].get(row).cloned().unwrap_or_default()
    }

    /// Cell parsed as `T`, or `None` if missing or unparsable.
    pub fn parse<T: FromStr>(&self, col: usize, row: usize) -> Option<T> {
        self.0[col].get(row)?.parse().ok()
    }
}

/// Rerun view class rendering an `S` table.
pub struct TableView<S>(PhantomData<S>);

impl<S> Default for TableView<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Visualizer feeding a [`TableView`].
pub struct TableSystem<S>(PhantomData<S>);

impl<S> Default for TableSystem<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Rows produced by [`TableSystem`], stored in [`VisualizerExecutionOutput`].
pub struct TableData<S: TableSpec>(Vec<S::Row>);

struct TableState {
    sort_col: usize,
    ascending: bool,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            sort_col: 0,
            ascending: true,
        }
    }
}

impl ViewState for TableState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn heap_size_bytes(&self) -> u64 {
        0
    }
}

impl<S: TableSpec> ViewClass for TableView<S> {
    fn identifier() -> ViewClassIdentifier
    where
        Self: Sized,
    {
        S::NAME.into()
    }

    fn display_name(&self) -> &'static str {
        S::NAME
    }

    fn icon(&self) -> &'static re_ui::Icon {
        S::ICON
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> re_ui::Help {
        re_ui::Help::new(format!("{} View", S::NAME))
    }

    fn on_register(
        &self,
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<TableSystem<S>>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<TableState>::default()
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
    ) -> Result<ViewClassUiOutput, ViewSystemExecutionError> {
        let tokens = ui.tokens();
        let state = state.downcast_mut::<TableState>()?;
        let rows: &[S::Row] = system_output
            .visualizer_data::<TableData<S>>(TableSystem::<S>::identifier())?
            .map(|d| d.0.as_slice())
            .unwrap_or_default();

        if rows.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.weak(S::EMPTY);
            });
            return Ok(ViewClassUiOutput::default());
        }

        let mut sorted: Vec<&S::Row> = rows.iter().collect();
        sorted.sort_by(|a, b| S::cmp(a, b, state.sort_col));
        if !state.ascending {
            sorted.reverse();
        }

        let table_style = re_ui::TableStyle::Dense;
        let (sort_col, ascending) = (state.sort_col, state.ascending);
        let last = S::COLUMNS.len() - 1;
        let mut clicked = None;

        egui::Frame {
            inner_margin: tokens.view_padding().into(),
            ..egui::Frame::default()
        }
        .show(ui, |ui| {
            let mut table = egui_extras::TableBuilder::new(ui)
                .resizable(true)
                .vscroll(true)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0)
                .max_scroll_height(f32::INFINITY)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
            for (i, &(_, min)) in S::COLUMNS.iter().enumerate() {
                table = table.column(if i == last {
                    Column::remainder().at_least(min)
                } else {
                    Column::auto().at_least(min).clip(true)
                });
            }
            table
                .header(tokens.deprecated_table_header_height(), |mut header| {
                    re_ui::DesignTokens::setup_table_header(&mut header);
                    for (i, &(label, _)) in S::COLUMNS.iter().enumerate() {
                        header.col(|ui| {
                            crate::ui::sortable_header(
                                ui,
                                label,
                                sort_col == i,
                                ascending,
                                &mut clicked,
                                i,
                            );
                        });
                    }
                })
                .body(|mut body| {
                    tokens.setup_table_body(&mut body, table_style);
                    let row_height = tokens.table_row_height(table_style);
                    body.rows(row_height, sorted.len(), |mut row| {
                        let entry = sorted[row.index()];
                        for i in 0..S::COLUMNS.len() {
                            row.col(|ui| S::cell(ui, entry, i));
                        }
                    });
                });
        });

        if let Some(col) = clicked {
            state.ascending = col != state.sort_col || !state.ascending;
            state.sort_col = col;
        }

        Ok(ViewClassUiOutput::default())
    }
}

impl<S: TableSpec> IdentifiedViewSystem for TableSystem<S> {
    fn identifier() -> ViewSystemIdentifier {
        S::NAME.into()
    }
}

impl<S: TableSpec> VisualizerSystem for TableSystem<S> {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo::empty()
    }

    fn execute(
        &self,
        ctx: &ViewContext<'_>,
        _query: &ViewQuery<'_>,
        _context_systems: &re_viewer_context::ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let ids: Vec<_> = S::descriptors().iter().map(|d| d.component).collect();
        let results = ctx
            .viewer_ctx
            .recording()
            .storage_engine()
            .cache()
            .latest_at(
                re_chunk_store::ChunkTrackingMode::Ignore,
                &LatestAtQuery::latest(TimelineName::log_time()),
                &EntityPath::from(S::ENTITY_PATH),
                ids.iter().copied(),
            );
        let cols = Columns(
            ids.iter()
                .map(|&id| {
                    results
                        .component_batch_raw(id)
                        .map_or_else(Vec::new, |arr| extract_texts(&arr))
                })
                .collect(),
        );
        Ok(VisualizerExecutionOutput::default()
            .with_visualizer_data(TableData::<S>(S::rows(&cols))))
    }
}

fn extract_texts(array: &dyn arrow::array::Array) -> Vec<String> {
    array
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .map(|s| s.iter().map(|v| v.unwrap_or_default().to_owned()).collect())
        .unwrap_or_default()
}
