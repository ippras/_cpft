use crate::r#const::EM_DASH;
use egui::{
    ComboBox, DragValue, Grid, PopupCloseBehavior, Response, ScrollArea, TextStyle, TextWrapMode,
    Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_extras::{Column, TableBuilder};
use egui_l10n::ContextExt as _;
use egui_phosphor::regular::SORT_ASCENDING;
use lipid::prelude::*;
use std::{borrow::Cow, cmp::Ordering, convert::identity};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    fatty_acid: &'a mut FattyAcid,
    hover: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(fatty_acid: &'a mut FattyAcid) -> Self {
        Self {
            fatty_acid,
            hover: false,
        }
    }

    pub(crate) fn hover(self, hover: bool) -> Self {
        Self { hover, ..self }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn show(self, ui: &mut Ui) -> Response {
        let mut inner = None;
        let text = self.fatty_acid.delta().to_string();
        let mut changed = false;
        let mut response = MenuButton::new(&text)
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                if content(self.fatty_acid)(ui).changed() {
                    inner = Some(self.fatty_acid);
                    changed = true;
                }
            })
            .0;
        if changed {
            response.mark_changed();
        };
        if self.hover {
            response = response.on_hover_text(&text);
        }
        response
    }
}

impl Widget for FattyAcidWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

fn content(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let response = ui
            .horizontal(|ui| {
                let mut response = ui.button(SORT_ASCENDING);
                if response.clicked() {
                    fatty_acid
                        .unsaturated
                        .sort_by_cached_key(|unsaturated| unsaturated.index);
                    response.mark_changed();
                }
                response |= carbon(fatty_acid)(ui);
                response |= unsaturated(fatty_acid)(ui);
                response
            })
            .inner;
        ui.separator();
        response | new_indices(fatty_acid)(ui)
    }
}

fn carbon(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        ui.label("Carbon");
        ui.add(DragValue::new(&mut fatty_acid.carbon).update_while_editing(false))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("carbon.hover"));
            })
    }
}

fn unsaturated(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        ui.label("Unsaturated");
        let mut unsaturated = fatty_acid.unsaturated.len();
        let response = ui
            .add(
                DragValue::new(&mut unsaturated)
                    .clamp_existing_to_range(true)
                    .range(0..=fatty_acid.carbon.saturating_sub(1))
                    .update_while_editing(false),
            )
            .on_hover_ui(|ui| {
                ui.label(ui.localize("unsaturated.hover"));
            });
        if response.changed() {
            loop {
                match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                    Ordering::Less => {
                        fatty_acid.unsaturated.pop();
                    }
                    Ordering::Equal => break,
                    Ordering::Greater => {
                        fatty_acid.unsaturated.push(Unsaturated {
                            index: Some(0),
                            parity: Some(false),
                            triple: Some(false),
                        });
                    }
                }
            }
        }
        response
    }
}

fn new_indices(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let mut response = ui.response();
        let height = ui.text_style_height(&TextStyle::Body);
        let width = ui.spacing().combo_width;
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
        TableBuilder::new(ui)
            .column(Column::auto().at_least(width / 2.0))
            .column(Column::auto().at_least(width))
            .column(Column::auto().at_least(width / 2.0))
            .header(0.0, |_| {})
            .body(|mut body| {
                for unsaturated in &mut fatty_acid.unsaturated {
                    body.row(height, |mut row| {
                        // Index
                        row.col(|ui| {
                            let text = match unsaturated.index {
                                Some(index) => Cow::Owned(index.to_string()),
                                None => Cow::Borrowed("?"),
                            };
                            let inner_response = ui.menu_button(text, |ui| {
                                ui.set_min_width(ui.spacing().combo_width / 2.0);
                                ui.set_max_height(ui.spacing().combo_height);
                                let mut changed = false;
                                ScrollArea::vertical().show(ui, |ui| {
                                    for selected in 1..fatty_acid.carbon {
                                        changed |= ui
                                            .selectable_value(
                                                &mut unsaturated.index,
                                                Some(selected),
                                                selected.to_string(),
                                            )
                                            .changed();
                                    }
                                    changed |= ui
                                        .selectable_value(&mut unsaturated.index, None, "?")
                                        .changed();
                                });
                                changed
                            });
                            response |= inner_response.response;
                            if inner_response.inner.is_some_and(identity) {
                                response.mark_changed();
                            }
                        });
                        // Triple
                        row.col(|ui| {
                            let text = match unsaturated.triple {
                                Some(false) => "Olefinic",
                                Some(true) => "Acetylenic",
                                None => EM_DASH,
                            };
                            let inner_response = ui.menu_button(text, |ui| {
                                let mut changed = false;
                                changed |= ui
                                    .selectable_value(
                                        &mut unsaturated.triple,
                                        Some(false),
                                        "Olefinic",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut unsaturated.triple,
                                        Some(true),
                                        "Acetylenic",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut unsaturated.triple, None, "?")
                                    .changed();
                                changed
                            });
                            response |= inner_response.response;
                            if inner_response.inner.is_some_and(identity) {
                                response.mark_changed();
                            }
                        });
                        // Parity
                        row.col(|ui| {
                            if let Some(false) = unsaturated.triple {
                                let text = match unsaturated.parity {
                                    Some(false) => "Cis",
                                    Some(true) => "Trans",
                                    None => EM_DASH,
                                };
                                let inner_response = ui.menu_button(text, |ui| {
                                    let mut changed = false;
                                    changed |= ui
                                        .selectable_value(
                                            &mut unsaturated.parity,
                                            Some(false),
                                            "Cis",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut unsaturated.parity,
                                            Some(true),
                                            "Trans",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(&mut unsaturated.parity, None, "?")
                                        .changed();
                                    changed
                                });
                                response |= inner_response.response;
                                if inner_response.inner.is_some_and(identity) {
                                    response.mark_changed();
                                }
                            }
                        });
                    });
                }
            });
        response
    }
}

fn indices(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let mut response = ui.response();
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            for unsaturated in &mut fatty_acid.unsaturated {
                // Index
                let text = match unsaturated.index {
                    Some(index) => Cow::Owned(index.to_string()),
                    None => Cow::Borrowed("None"),
                };
                let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Index"))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        for selected in 1..fatty_acid.carbon {
                            changed |= ui
                                .selectable_value(
                                    &mut unsaturated.index,
                                    Some(selected),
                                    selected.to_string(),
                                )
                                .changed();
                        }
                        changed |= ui
                            .selectable_value(&mut unsaturated.index, None, "None")
                            .changed();
                        changed
                    });
                response |= inner_response.response;
                if inner_response.inner.is_some_and(identity) {
                    response.mark_changed();
                }
                // Triple
                let text = match unsaturated.triple {
                    Some(false) => "Olefinic",
                    Some(true) => "Acetylenic",
                    None => "None",
                };
                let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Triple"))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, Some(false), "Olefinic")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, Some(true), "Acetylenic")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, None, "None")
                            .changed();
                        changed
                    });
                response |= inner_response.response;
                if inner_response.inner.is_some_and(identity) {
                    response.mark_changed();
                }
                if let Some(false) = unsaturated.triple {
                    // Parity
                    let text = match unsaturated.parity {
                        Some(false) => "Cis",
                        Some(true) => "Trans",
                        None => "None",
                    };
                    let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Parity"))
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .selected_text(text)
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, Some(false), "Cis")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, Some(true), "Trans")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, None, "None")
                                .changed();
                            changed
                        });
                    response |= inner_response.response;
                    if inner_response.inner.is_some_and(identity) {
                        response.mark_changed();
                    }
                }
                ui.end_row();
            }
        });
        response
    }
}
