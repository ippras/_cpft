use crate::{
    app::{MAX_PRECISION, panes::source::table::NUM_COLUMNS, states::source::ID_SOURCE},
    localization::Text,
    utils::VecExt as _,
};
use egui::{
    ComboBox, Grid, Popup, PopupCloseBehavior, Slider, TextWrapMode, Ui, Vec2b, Widget,
    emath::Float as _,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{BOOKMARK, FUNNEL, FUNNEL_X};
use lipid::prelude::FattyAcid;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

const MARGARIC: FattyAcid = FattyAcid {
    carbon: 17,
    unsaturated: Vec::new(),
};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) significant: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) view: View,
    pub(crate) ddof: u8,
    pub(crate) logarithmic: bool,
    pub(crate) relative: Option<FattyAcid>,
    pub(crate) filter: Filter,
    pub(crate) order: Order,
    pub(crate) sort: Sort,

    pub(crate) plot: Plot,

    pub(crate) cache: Cache,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            significant: true,
            sticky: 0,
            truncate: false,

            view: View::Table,
            ddof: 1,
            logarithmic: false,
            relative: None,
            filter: Filter::new(),
            sort: Sort::Time,
            order: Order::Ascending,

            plot: Plot::new(),

            cache: Cache::new(),
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        Grid::new(ui.auto_id_with(ID_SOURCE))
            .show(ui, |ui| -> PolarsResult<()> {
                self.precision(ui);
                self.significant(ui);
                self.sticky(ui);
                self.truncate(ui);

                // Calculate
                ui.heading("Calculate");
                ui.separator();
                ui.end_row();

                self.relative(ui);

                // DDOF
                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                    .on_hover_localized("DeltaDegreesOfFreedom")
                    .on_hover_ui(|ui| {
                        ui.hyperlink(
                            "https://numpy.org/devdocs/reference/generated/numpy.std.html",
                        );
                    });
                Slider::new(&mut self.ddof, 0..=2).ui(ui);
                ui.end_row();

                // Logarithmic
                ui.label(ui.localize("LogarithmOfTheRetentionTime"))
                    .on_hover_localized("LogarithmOfTheRetentionTime.hover");
                ui.checkbox(&mut self.logarithmic, "");
                ui.end_row();

                // Filter
                ui.heading("Filter");
                ui.separator();
                ui.end_row();

                self.filter(ui);

                // Sort, order
                ui.heading("SortBy");
                ui.separator();
                ui.end_row();

                self.sort(ui);
                self.order(ui);
                if let View::Plot = self.view {
                    // Plot
                    ui.heading("Plot");
                    ui.separator();
                    ui.end_row();

                    // // Group
                    // ui.label("Group");
                    // ComboBox::from_id_salt(ui.next_auto_id())
                    //     .selected_text(self.group.text())
                    //     .show_ui(ui, |ui| {
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::FattyAcid,
                    //             Group::FattyAcid.text(),
                    //         )
                    //         .on_hover_text(Group::FattyAcid.hover_text());
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::OnsetTemperature,
                    //             Group::OnsetTemperature.text(),
                    //         )
                    //         .on_hover_text(Group::OnsetTemperature.hover_text());
                    //         ui.selectable_value(
                    //             &mut self.group,
                    //             Group::TemperatureStep,
                    //             Group::TemperatureStep.text(),
                    //         )
                    //         .on_hover_text(Group::TemperatureStep.hover_text());
                    //     })
                    //     .response
                    //     .on_hover_text(self.group.hover_text());
                    // ui.end_row();
                    self.legend(ui);
                    self.radius_of_points(ui);
                }
                Ok(())
            })
            .inner
    }

    /// Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Precision"))
            .on_hover_localized("Precision.hover");
        ui.horizontal(|ui| {
            Slider::new(&mut self.precision, 1..=MAX_PRECISION).ui(ui);
            if ui.button((BOOKMARK, "3")).clicked() {
                self.precision = 3;
            };
        });
        ui.end_row();
    }

    /// Significant
    fn significant(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Significant"))
            .on_hover_localized("Significant.hover");
        ui.checkbox(&mut self.significant, ());
        ui.end_row();
    }

    // Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StickyColumns"))
            .on_hover_localized("StickyColumns.hover");
        Slider::new(&mut self.sticky, 0..=NUM_COLUMNS).ui(ui);
        ui.end_row();
    }

    // Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("TruncateHeaders"))
            .on_hover_localized("TruncateHeaders.hover");
        ui.checkbox(&mut self.truncate, "");
        ui.end_row();
    }

    /// Relative
    fn relative(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("RelativeFattyAcid"))
            .on_hover_localized("RelativeFattyAcid.hover");
        ui.horizontal(|ui| {
            let selected_text = self
                .relative
                .as_ref()
                .map(|fatty_acid| fatty_acid.delta().to_string())
                .unwrap_or_default();
            ComboBox::from_id_salt(ui.auto_id_with("Relative"))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    let current_value = &mut self.relative;
                    let saturated_fatty_acids = self
                        .cache
                        .fatty_acids
                        .iter()
                        .filter(|fatty_acid| fatty_acid.unsaturated.is_empty());
                    for selected_value in saturated_fatty_acids {
                        ui.selectable_value(
                            current_value,
                            Some(selected_value.clone()),
                            selected_value.delta().to_string(),
                        );
                    }
                    ui.selectable_value(current_value, None, "-");
                });
            if ui
                .button((BOOKMARK, "17:0"))
                .on_hover_text("Margaric")
                .clicked()
            {
                self.relative = Some(MARGARIC);
            };
        });
        ui.end_row();
    }

    /// Filter
    fn filter(&mut self, ui: &mut Ui) {
        // Onset temperature filter
        ui.label(ui.localize("FilterByOnsetTemperature"))
            .on_hover_localized("FilterByOnsetTemperature.hover");
        let text = format_list_truncated!(&self.filter.onset_temperatures, 1);
        let response = ComboBox::from_id_salt("OnsetTemperatureFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| {
                for &onset_temperature in &self.cache.onset_temperatures {
                    let mut checked = !self.filter.onset_temperatures.contains(&onset_temperature);
                    if ui
                        .checkbox(&mut checked, onset_temperature.to_string())
                        .changed()
                    {
                        if checked {
                            self.filter
                                .onset_temperatures
                                .remove_by_value(&onset_temperature);
                        } else {
                            self.filter.onset_temperatures.push(onset_temperature);
                        }
                    }
                }
            })
            .response;
        Popup::context_menu(&response)
            .id(ui.next_auto_id().with("ContextMenu"))
            .show(|ui| {
                if ui.button((FUNNEL, ui.localize("CheckAll"))).clicked() {
                    self.filter.onset_temperatures = Vec::new();
                }
                if ui.button((FUNNEL_X, ui.localize("UncheckAll"))).clicked() {
                    self.filter.onset_temperatures = self.cache.onset_temperatures.clone();
                }
            });
        ui.end_row();

        // Temperature step filter
        ui.label(ui.localize("FilterByTemperatureStep"))
            .on_hover_localized("FilterByTemperatureStep.hover");
        let text = format_list_truncated!(&self.filter.temperature_steps, 1);
        let response = ComboBox::from_id_salt("TemperatureStepFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| {
                for &temperature_step in &self.cache.temperature_steps {
                    let mut checked = !self.filter.temperature_steps.contains(&temperature_step);
                    if ui
                        .checkbox(&mut checked, temperature_step.to_string())
                        .changed()
                    {
                        if checked {
                            self.filter
                                .temperature_steps
                                .remove_by_value(&temperature_step);
                        } else {
                            self.filter.temperature_steps.push(temperature_step);
                        }
                    }
                }
            })
            .response;
        Popup::context_menu(&response)
            .id(ui.next_auto_id().with("ContextMenu"))
            .show(|ui| {
                if ui.button((FUNNEL, ui.localize("CheckAll"))).clicked() {
                    self.filter.temperature_steps = Vec::new();
                }
                if ui.button((FUNNEL_X, ui.localize("UncheckAll"))).clicked() {
                    self.filter.temperature_steps = self.cache.temperature_steps.clone();
                }
            });
        ui.end_row();

        // Fatty acids filter
        ui.label(ui.localize("FilterByFattyAcids"))
            .on_hover_localized("FilterByFattyAcids.hover");
        let text = format_list_truncated!(
            self.filter
                .fatty_acids
                .iter()
                .map(|fatty_acid| fatty_acid.delta()),
            1
        );
        let mut response = ComboBox::from_id_salt("FattyAcidsFilter")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| {
                for fatty_acid in &self.cache.fatty_acids {
                    let mut checked = !self.filter.fatty_acids.contains(&fatty_acid);
                    if ui
                        .checkbox(&mut checked, fatty_acid.delta().to_string())
                        .changed()
                    {
                        if checked {
                            self.filter.fatty_acids.remove_by_value(fatty_acid);
                        } else {
                            self.filter.fatty_acids.push(fatty_acid.clone());
                        }
                    }
                }
            })
            .response;
        response = response.on_hover_ui(|ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.label(format!(
                "{}/{}",
                self.filter.fatty_acids.len(),
                self.cache.fatty_acids.len()
            ));
        });
        Popup::context_menu(&response)
            .id(ui.next_auto_id().with("ContextMenu"))
            .show(|ui| {
                if ui.button((FUNNEL, ui.localize("CheckAll"))).clicked() {
                    self.filter.fatty_acids = Vec::new();
                }
                if ui.button((FUNNEL_X, ui.localize("UncheckAll"))).clicked() {
                    self.filter.fatty_acids = self.cache.fatty_acids.clone();
                }
            });
        ui.end_row();
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("SortBy"))
            .on_hover_localized("SortBy.hover");
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(ui.localize(self.sort.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.sort,
                    Sort::FattyAcid,
                    ui.localize(Sort::FattyAcid.text()),
                )
                .on_hover_localized(Sort::FattyAcid.hover_text());
                ui.selectable_value(&mut self.sort, Sort::Time, ui.localize(Sort::Time.text()))
                    .on_hover_localized(Sort::Time.hover_text());
            })
            .response
            .on_hover_localized(self.sort.hover_text());
        ui.end_row();
    }

    /// Order
    fn order(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Order"))
            .on_hover_localized("Order.hover");
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(ui.localize(self.order.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.order,
                    Order::Ascending,
                    ui.localize(Order::Ascending.text()),
                )
                .on_hover_localized(Order::Ascending.hover_text());
                ui.selectable_value(
                    &mut self.order,
                    Order::Descending,
                    ui.localize(Order::Descending.text()),
                )
                .on_hover_localized(Order::Descending.hover_text());
            })
            .response
            .on_hover_localized(self.order.hover_text());
        ui.end_row();
    }

    /// Legend
    fn legend(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Legend"));
        ui.checkbox(&mut self.plot.legend, "");
        ui.end_row();
    }

    /// Radius of points
    fn radius_of_points(&mut self, ui: &mut Ui) {
        // Radius of points
        ui.label(ui.localize("RadiusOfPoints"))
            .on_hover_localized("RadiusOfPoints.hover");
        ui.horizontal(|ui| {
            Slider::new(&mut self.plot.radius_of_points, 0..=u8::MAX)
                .logarithmic(true)
                .ui(ui);
            if ui.button((BOOKMARK, "2")).clicked() {
                self.plot.radius_of_points = 2;
            };
        });
        ui.end_row();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Cache {
    pub(crate) onset_temperatures: Vec<f64>,
    pub(crate) temperature_steps: Vec<f64>,
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            onset_temperatures: Vec::new(),
            temperature_steps: Vec::new(),
            fatty_acids: Vec::new(),
        }
    }
}

impl Hash for Cache {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for onset_temperature in &self.onset_temperatures {
            onset_temperature.ord().hash(state);
        }
        for temperature_step in &self.temperature_steps {
            temperature_step.ord().hash(state);
        }
        self.fatty_acids.hash(state);
    }
}

/// Plot
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Plot {
    pub(crate) drag: Vec2b,
    pub(crate) legend: bool,
    pub(crate) radius_of_points: u8,
    pub(crate) scroll: Vec2b,
}

impl Plot {
    pub(crate) fn new() -> Self {
        Self {
            drag: Vec2b::TRUE,
            legend: true,
            radius_of_points: 2,
            scroll: Vec2b::TRUE,
        }
    }
}

impl Hash for Plot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.drag.x.hash(state);
        self.drag.y.hash(state);
        self.legend.hash(state);
        self.radius_of_points.hash(state);
        self.scroll.x.hash(state);
        self.scroll.y.hash(state);
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    FattyAcid,
    Time,
}

impl Text for Sort {
    fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "SortByFattyAcids",
            Self::Time => "SortByRetentionTime",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "SortByFattyAcids.hover",
            Self::Time => "SortByRetentionTime.hover",
        }
    }
}

/// Plot settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct PlotSettings {
    pub(crate) legend: bool,
    pub(crate) radius_of_points: u8,
    pub(crate) axes: Axes,
}

impl PlotSettings {
    pub(crate) fn new() -> Self {
        Self {
            radius_of_points: 2,
            legend: true,
            axes: Axes {
                x: Axis::TemperatureStep,
                y: Axis::Alpha,
            },
        }
    }
}

// Plot axes
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Axes {
    pub(crate) x: Axis,
    pub(crate) y: Axis,
}

/// Plot axis
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Axis {
    Alpha,
    EquivalentChainLength,
    OnsetTemperature,
    TemperatureStep,
}

impl Text for Axis {
    fn text(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::EquivalentChainLength => "EquivalentChainLength",
            Self::OnsetTemperature => "OnsetTemperature",
            Self::TemperatureStep => "TemperatureStep",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha.hover",
            Self::EquivalentChainLength => "EquivalentChainLength.hover",
            Self::OnsetTemperature => "OnsetTemperature.hover",
            Self::TemperatureStep => "TemperatureStep.hover",
        }
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) onset_temperatures: Vec<f64>,
    pub(crate) temperature_steps: Vec<f64>,
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub(crate) fn new() -> Self {
        Self {
            onset_temperatures: Vec::new(),
            temperature_steps: Vec::new(),
            fatty_acids: Vec::new(),
        }
    }
}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for onset_temperature in &self.onset_temperatures {
            onset_temperature.ord().hash(state);
        }
        for temperature_step in &self.temperature_steps {
            temperature_step.ord().hash(state);
        }
        self.fatty_acids.hash(state);
    }
}

// impl Filter {
//     pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
//         // Onset temperature filter
//         ui.label(ui.localize("filter-by-onset-temperature"))
//             .on_hover_localized("filter-by-onset-temperature.hover");
//         let text = format_list_truncated!(&self.onset_temperatures, 2);
//         ComboBox::from_id_salt("OnsetTemperatureFilter")
//             .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
//             .selected_text(text)
//             .show_ui(ui, |ui| -> PolarsResult<()> {
//                 let onset_temperatures = data_frame.mode().onset_temperature()?.unique();
//                 for onset_temperature in onset_temperatures.iter().flatten() {
//                     let checked = self.onset_temperatures.contains(&onset_temperature);
//                     let response =
//                         ui.selectable_label(checked, AnyValue::from(onset_temperature).to_string());
//                     if response.clicked() {
//                         if checked {
//                             self.onset_temperatures.remove_by_value(&onset_temperature);
//                         } else {
//                             self.onset_temperatures.push(onset_temperature);
//                         }
//                     }
//                     response.context_menu(|ui| {
//                         if ui.button(format!("{FUNNEL} Select all")).clicked() {
//                             self.onset_temperatures = onset_temperatures.iter().flatten().collect();
//                             ui.close_kind(UiKind::Menu);
//                         }
//                         if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
//                             self.onset_temperatures = Vec::new();
//                             ui.close_kind(UiKind::Menu);
//                         }
//                     });
//                 }
//                 Ok(())
//             })
//             .inner
//             .transpose()?;
//         ui.end_row();

//         // Temperature step filter
//         ui.label(ui.localize("filter-by-temperature-step"))
//             .on_hover_localized("filter-by-temperature-step.hover");
//         let text = format_list_truncated!(&self.temperature_steps, 2);
//         ComboBox::from_id_salt("TemperatureStepFilter")
//             .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
//             .selected_text(text)
//             .show_ui(ui, |ui| -> PolarsResult<()> {
//                 let temperature_steps = data_frame.mode().temperature_step()?.unique();
//                 for temperature_step in temperature_steps.iter().flatten() {
//                     let checked = self.temperature_steps.contains(&temperature_step);
//                     let response =
//                         ui.selectable_label(checked, AnyValue::from(temperature_step).to_string());
//                     if response.clicked() {
//                         if checked {
//                             self.temperature_steps.remove_by_value(&temperature_step);
//                         } else {
//                             self.temperature_steps.push(temperature_step);
//                         }
//                     }
//                     response.context_menu(|ui| {
//                         if ui.button(format!("{FUNNEL} Select all")).clicked() {
//                             self.temperature_steps = temperature_steps.iter().flatten().collect();
//                             ui.close_kind(UiKind::Menu);
//                         }
//                         if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
//                             self.temperature_steps = Vec::new();
//                             ui.close_kind(UiKind::Menu);
//                         }
//                     });
//                 }
//                 Ok(())
//             })
//             .inner
//             .transpose()?;
//         ui.end_row();

//         // Fatty acids filter
//         ui.label(ui.localize("filter-by-fatty-acids"))
//             .on_hover_localized("filter-by-fatty-acids.hover");
//         let text = format_list_truncated!(
//             self.fatty_acids.iter().map(|fatty_acid| fatty_acid.delta()),
//             2
//         );
//         let inner_response = ComboBox::from_id_salt("FattyAcidsFilter")
//             .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
//             .selected_text(text)
//             .show_ui(ui, |ui| -> PolarsResult<()> {
//                 let fatty_acids = data_frame["FattyAcid"]
//                     .unique()?
//                     .sort(Default::default())?
//                     .fatty_acid();
//                 // for index in 0..fatty_acids.len() {
//                 //     let Some(fatty_acid) = fatty_acids.get(index)? else {
//                 //         continue;
//                 //     };
//                 //     let checked = self.fatty_acids.contains(&fatty_acid);
//                 //     let response =
//                 //         ui.selectable_label(checked, format!("{:#}", (&fatty_acid).delta()));
//                 //     if response.clicked() {
//                 //         if checked {
//                 //             self.fatty_acids.remove_by_value(&fatty_acid);
//                 //         } else {
//                 //             self.fatty_acids.push(fatty_acid);
//                 //         }
//                 //     }
//                 //     response.context_menu(|ui| {
//                 //         if ui.button(format!("{FUNNEL} Select all")).clicked() {
//                 //             // self.fatty_acids = fatty_acids.clone().into_iter().flatten().collect();
//                 //             ui.close_kind(UiKind::Menu);
//                 //         }
//                 //         if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
//                 //             self.fatty_acids = Vec::new();
//                 //             ui.close_kind(UiKind::Menu);
//                 //         }
//                 //     });
//                 // }
//                 Ok(())
//             });
//         inner_response.inner.transpose()?;
//         inner_response.response.on_hover_ui(|ui| {
//             ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
//             ui.label(self.fatty_acids.len().to_string());
//         });
//         Ok(())
//     }
// }

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Text for Order {
    fn text(&self) -> &'static str {
        match self {
            Self::Ascending => "AscendingOrder",
            Self::Descending => "DescendingOrder",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Ascending => "AscendingOrder.hover",
            Self::Descending => "DescendingOrder.hover",
        }
    }
}

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl Text for View {
    fn text(&self) -> &'static str {
        match self {
            Self::Plot => "PlotView",
            Self::Table => "TableView",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Plot => "TableView.hover",
            Self::Table => "PlotView.hover",
        }
    }
}
