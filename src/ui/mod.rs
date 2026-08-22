mod status_bar;

use re_ui::UiExt as _;

pub use status_bar::StatusBar;

/// Renders a clickable column header with a sort-direction indicator.
///
/// When clicked, sets `clicked` to `col` so the caller can toggle sort state.
pub fn sortable_header(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    ascending: bool,
    clicked: &mut Option<usize>,
    col: usize,
) {
    let text = egui::RichText::new(label).strong();
    let response = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

    if is_active {
        let icon = if ascending {
            &re_ui::icons::ARROW_UP
        } else {
            &re_ui::icons::ARROW_DOWN
        };
        ui.small_icon(icon, None);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(
            response.rect,
            2.0,
            egui::Color32::from_rgba_premultiplied(0, 61, 161, 30),
        );
    }

    if response.clicked() {
        *clicked = Some(col);
    }
}
