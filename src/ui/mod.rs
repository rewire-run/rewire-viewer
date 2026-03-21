mod status_bar;

use egui;

pub use status_bar::StatusBar;

pub fn sortable_header<C: Copy>(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    ascending: bool,
    clicked: &mut Option<C>,
    col: C,
) {
    let text = egui::RichText::new(label).strong();
    let response = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

    if is_active {
        let icon_size = 6.0;
        let icon_center = egui::pos2(
            response.rect.right() + 4.0 + icon_size * 0.5,
            response.rect.center().y,
        );
        let half_w = icon_size * 0.5;
        let half_h = icon_size * 0.35;
        let color = ui.visuals().text_color();

        let points = if ascending {
            vec![
                egui::pos2(icon_center.x - half_w, icon_center.y + half_h),
                egui::pos2(icon_center.x + half_w, icon_center.y + half_h),
                egui::pos2(icon_center.x, icon_center.y - half_h),
            ]
        } else {
            vec![
                egui::pos2(icon_center.x - half_w, icon_center.y - half_h),
                egui::pos2(icon_center.x + half_w, icon_center.y - half_h),
                egui::pos2(icon_center.x, icon_center.y + half_h),
            ]
        };

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::NONE,
        ));
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
