use crate::r#const::{
    EQUIVALENT_CHAIN_LENGTH, ONSET_TEMPERATURE, SELECTIVITY_FACTOR, TEMPERATURE_STEP,
};
use const_format::formatcp;
use egui::{ComboBox, Ui};
use egui_l10n::ContextExt as _;
use serde::{Deserialize, Serialize};
use widgets::settings::Plot as Control;

/// Plot
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Plot {
    pub(crate) axes: Axes,
    pub(crate) control: Control,
}

impl Plot {
    pub(crate) fn new() -> Self {
        Self {
            axes: Axes::new(),
            control: Control::new(),
        }
    }
}

impl Plot {
    pub fn show(&mut self, ui: &mut Ui) {
        self.axes.show(ui);
        self.control.show(ui);
    }
}

// Plot axes
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Axes {
    pub(crate) x: X,
    pub(crate) y: Y,
}

impl Axes {
    pub(crate) fn new() -> Self {
        Self {
            x: X::TemperatureStep,
            y: Y::SelectivityFactor,
        }
    }
}

impl Axes {
    pub fn show(&mut self, ui: &mut Ui) {
        self.x.show(ui);
        self.y.show(ui);
    }
}

/// Plot x axis
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum X {
    OnsetTemperature,
    TemperatureStep,
}

impl X {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("X");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, X::OnsetTemperature, X::OnsetTemperature.text())
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(X::OnsetTemperature.hover_text()));
                        });
                    ui.selectable_value(self, X::TemperatureStep, X::TemperatureStep.text())
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(X::TemperatureStep.hover_text()));
                        });
                })
                .response
                .on_hover_text(self.hover_text());
        });
    }
}

impl X {
    pub const fn text(&self) -> &'static str {
        match self {
            Self::OnsetTemperature => ONSET_TEMPERATURE,
            Self::TemperatureStep => TEMPERATURE_STEP,
        }
    }

    pub const fn hover_text(&self) -> &'static str {
        match self {
            Self::OnsetTemperature => formatcp!("{ONSET_TEMPERATURE}.hover"),
            Self::TemperatureStep => formatcp!("{TEMPERATURE_STEP}.hover"),
        }
    }
}

/// Plot y axis
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Y {
    SelectivityFactor,
    EquivalentChainLength,
}

impl Y {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Y");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Y::SelectivityFactor, Y::SelectivityFactor.text())
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(Y::SelectivityFactor.hover_text()));
                        });
                    ui.selectable_value(
                        self,
                        Y::EquivalentChainLength,
                        Y::EquivalentChainLength.text(),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Y::EquivalentChainLength.hover_text()));
                    });
                })
                .response
                .on_hover_text(self.hover_text());
        });
    }
}

impl Y {
    pub const fn text(&self) -> &'static str {
        match self {
            Self::SelectivityFactor => SELECTIVITY_FACTOR,
            Self::EquivalentChainLength => EQUIVALENT_CHAIN_LENGTH,
        }
    }

    pub const fn hover_text(&self) -> &'static str {
        match self {
            Self::SelectivityFactor => formatcp!("{SELECTIVITY_FACTOR}.hover"),
            Self::EquivalentChainLength => formatcp!("{EQUIVALENT_CHAIN_LENGTH}.hover"),
        }
    }
}
