use const_format::formatcp;
use egui::{ComboBox, Ui};
use egui_l10n::ContextExt;
use serde::{Deserialize, Serialize};
use widgets::r#const::SORT;

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Sort {
    pub kind: SortKind,
}

impl Sort {
    pub fn new() -> Self {
        Self {
            kind: SortKind::RetentionTime,
        }
    }
}

impl Sort {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize(SORT)).on_hover_ui(|ui| {
                ui.label(ui.localize(formatcp!("{SORT}.hover")));
            });
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(ui.localize(self.kind.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.kind,
                        SortKind::FattyAcid,
                        ui.localize(SortKind::FattyAcid.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(SortKind::FattyAcid.hover_text()));
                    });
                    ui.selectable_value(
                        &mut self.kind,
                        SortKind::RetentionTime,
                        ui.localize(SortKind::RetentionTime.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(SortKind::RetentionTime.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.kind.hover_text()));
                });
        });
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum SortKind {
    FattyAcid,
    RetentionTime,
}

impl SortKind {
    pub const fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "SortByFattyAcids",
            Self::RetentionTime => "SortByRetentionTime",
        }
    }

    pub const fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "SortByFattyAcids.hover",
            Self::RetentionTime => "SortByRetentionTime.hover",
        }
    }
}
