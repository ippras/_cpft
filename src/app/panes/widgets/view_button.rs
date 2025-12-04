use crate::{app::states::source::View, localization::Text as _};
use egui::{Response, RichText, Ui, UiKind, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{CHART_BAR, TABLE};

/// View button widget
#[derive(Debug)]
pub struct ViewButton<'a> {
    view: &'a mut View,
    size: Option<f32>,
}

impl<'a> ViewButton<'a> {
    pub fn new(view: &'a mut View) -> Self {
        Self { view, size: None }
    }

    pub fn with_size(self, size: Option<f32>) -> Self {
        Self { size: size, ..self }
    }
}

impl Widget for ViewButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = match self.view {
            View::Plot => CHART_BAR,
            View::Table => TABLE,
        };
        let mut atoms = RichText::new(text);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.menu_button(atoms, |ui| {
            let mut response = ui
                .selectable_value(self.view, View::Table, ui.localize(View::Table.text()))
                .on_hover_localized(View::Table.hover_text());
            response |= ui
                .selectable_value(self.view, View::Plot, ui.localize(View::Plot.text()))
                .on_hover_localized(View::Plot.hover_text());
            if response.changed() {
                ui.close_kind(UiKind::Menu);
            }
        })
        .response
    }
}
