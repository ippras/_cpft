use crate::{
    r#const::{
        DEAD_TIME, EQUIVALENT_CHAIN_LENGTH, FRACTIONAL_CHAIN_LENGTH, MASS, RETENTION_FACTOR,
        RETENTION_TIME, SELECTIVITY_FACTOR, TEMPERATURE,
    },
    localization::Text,
};
use const_format::formatcp;
use egui::{Popup, PopupCloseBehavior, RichText, Ui};
use egui_dnd::dnd;
use egui_l10n::prelude::*;
use egui_phosphor::regular::DOTS_SIX_VERTICAL;
use serde::{Deserialize, Serialize};
use std::slice::Iter;

const COLUMNS: [Column; 8] = [
    Column::DeadTime,
    Column::RetentionTime,
    Column::RetentionFactor,
    Column::SelectivityFactor,
    Column::EquivalentChainLength,
    Column::FractionalChainLength,
    Column::Temperature,
    Column::Mass,
];

/// Export
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Export(Vec<Item>);

impl Export {
    pub(crate) fn new() -> Self {
        Self(COLUMNS.map(Item::new).to_vec())
    }
}

impl Export {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let mut visible_all = None;
        let response = dnd(ui, ui.auto_id_with("Export")).show(
            self.0.iter_mut(),
            |ui, item, handle, _state| {
                ui.horizontal(|ui| {
                    let visible = item.visible;
                    handle.ui(ui, |ui| {
                        ui.label(DOTS_SIX_VERTICAL);
                    });
                    ui.checkbox(&mut item.visible, "");
                    let mut text = RichText::new(ui.localize(item.column.text()));
                    if !visible {
                        text = text.weak();
                    }
                    let response = ui.label(text);
                    Popup::context_menu(&response)
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            if ui.button("Show all").clicked() {
                                visible_all = Some(true);
                            }
                            if ui.button("Hide all").clicked() {
                                visible_all = Some(false);
                            }
                        });
                });
            },
        );
        if response.is_drag_finished() {
            response.update_vec(self.0.as_mut_slice());
        }
        if let Some(visible) = visible_all {
            for index in &mut self.0 {
                index.visible = visible;
            }
        }
    }
}

impl<'a> IntoIterator for &'a Export {
    type Item = &'a Item;
    type IntoIter = Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Export item
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Item {
    pub(crate) column: Column,
    pub(crate) visible: bool,
}

impl Item {
    pub(crate) fn new(column: Column) -> Self {
        Self {
            column,
            visible: true,
        }
    }
}

/// Column
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Column {
    DeadTime,
    RetentionTime,
    RetentionFactor,
    SelectivityFactor,
    EquivalentChainLength,
    FractionalChainLength,
    Temperature,
    Mass,
}

impl Text for Column {
    fn text(&self) -> &'static str {
        match self {
            Self::DeadTime => DEAD_TIME,
            Self::RetentionTime => RETENTION_TIME,
            Self::RetentionFactor => RETENTION_FACTOR,
            Self::SelectivityFactor => SELECTIVITY_FACTOR,
            Self::EquivalentChainLength => EQUIVALENT_CHAIN_LENGTH,
            Self::FractionalChainLength => FRACTIONAL_CHAIN_LENGTH,
            Self::Temperature => TEMPERATURE,
            Self::Mass => MASS,
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::DeadTime => formatcp!("{DEAD_TIME}.hover"),
            Self::RetentionTime => formatcp!("{RETENTION_TIME}.hover"),
            Self::RetentionFactor => formatcp!("{RETENTION_FACTOR}.hover"),
            Self::SelectivityFactor => formatcp!("{SELECTIVITY_FACTOR}.hover"),
            Self::EquivalentChainLength => formatcp!("{EQUIVALENT_CHAIN_LENGTH}.hover"),
            Self::FractionalChainLength => formatcp!("{FRACTIONAL_CHAIN_LENGTH}.hover"),
            Self::Temperature => formatcp!("{TEMPERATURE}.hover"),
            Self::Mass => formatcp!("{MASS}.hover"),
        }
    }
}
