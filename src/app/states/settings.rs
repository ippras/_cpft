use crate::app::ID_SOURCE;
use egui::{ComboBox, Grid, Sense, Ui};
use egui_l10n::prelude::*;
use egui_tiles::ContainerKind;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) layout: Layout,
    pub(crate) reactive: bool,
    pub(crate) reset_state: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            layout: Layout::new(),
            reactive: true,
            reset_state: false,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new(ID_SOURCE).show(ui, |ui| {
            // Language
            ui.label(ui.localize("Language"));
            let mut current_value = ui.language_identifier();
            ComboBox::from_id_salt(ui.auto_id_with("Language"))
                .selected_text(current_value.to_string())
                .show_ui(ui, |ui| {
                    let mut response = ui.allocate_response(Default::default(), Sense::click());
                    for selected_value in ui.language_identifiers() {
                        let text = selected_value.to_string();
                        response |= ui.selectable_value(&mut current_value, selected_value, text);
                    }
                    if response.changed() {
                        ui.set_language_identifier(current_value);
                    }
                });
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Layout {
    pub(crate) container_kind: Option<ContainerKind>,
}

impl Layout {
    fn new() -> Self {
        Self {
            container_kind: None,
        }
    }
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.container_kind
            .map(|container_kind| container_kind as usize)
            .hash(state);
    }
}
