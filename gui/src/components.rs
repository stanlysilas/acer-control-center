use eframe::egui::{self, Color32, RichText};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    Auto,
    Max,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    Off,
    Breathing,
}

pub fn segmented_choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: T,
    options: &[(T, &str)],
) -> Option<T> {
    let mut choice = None;
    let width = (ui.available_width() / options.len() as f32).clamp(110.0, 190.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (value, label) in options {
            let selected = current == *value;
            let text = if selected {
                RichText::new(*label).strong().color(Color32::from_rgb(207, 236, 231))
            } else {
                RichText::new(*label).color(Color32::from_rgb(190, 197, 204))
            };
            let fill = if selected {
                Color32::from_rgb(38, 59, 59)
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };
            let button = egui::Button::new(text)
                .selected(selected)
                .fill(fill)
                .stroke(if selected { egui::Stroke::new(1.0_f32, Color32::from_rgb(78, 129, 124)) } else { ui.visuals().widgets.inactive.bg_stroke })
                .corner_radius(7.0)
                .min_size(egui::vec2(width, 42.0));
            if ui.add_sized([width, 42.0], button).clicked() {
                choice = Some(*value);
            }
        }
    });
    choice
}

pub fn sidebar_button(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, 40.0),
        egui::Sense::click(),
    );
    let fill = if selected {
        Color32::from_rgb(37, 48, 49)
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
            2.0,
            Color32::from_rgb(95, 174, 166),
        );
    }
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::TextStyle::Button.resolve(ui.style()).clone(),
        if selected { Color32::from_rgb(207, 236, 231) } else { Color32::from_rgb(190, 197, 204) },
    );
    response.clicked()
}
