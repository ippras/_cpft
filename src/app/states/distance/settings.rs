use crate::{
    app::{
        MAX_PRECISION,
        panes::distance::view::table::NUM_COLUMNS,
        states::source::{Axis, Filter, Order, PlotSettings, View},
    },
    r#const::{ALPHA, EQUIVALENT_CHAIN_LENGTH, EUCLIDEAN, MAXIMUM, MEAN, MEDIAN, MINIMUM},
    localization::Text,
};
use egui::{ComboBox, RichText, Slider, Ui, Widget as _};
use egui_l20n::prelude::*;
use egui_phosphor::regular::BOOKMARK;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

const AGGREGATIONS: [Aggregation; 4] = [
    Aggregation::Maximum,
    Aggregation::Mean,
    Aggregation::Median,
    Aggregation::Minimum,
];

const DISTANCES: [Distance; 3] = [
    Distance::Alpha,
    Distance::EquivalentChainLength,
    Distance::Euclidean,
];

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) ddof: u8,
    pub(crate) mean: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) significant: bool,
    pub(crate) standard_deviation: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) filter: Filter,
    pub(crate) sort: Sort,
    pub(crate) order: Order,
    pub(crate) priority: Priority,

    pub(crate) view: View,
    pub(crate) plot: PlotSettings,
    // Reset
    pub(crate) reset_sum: bool,
    pub(crate) reset_table: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            precision: 3,
            resizable: false,
            significant: false,
            sticky: 0,
            truncate: false,

            ddof: 1,
            mean: false,
            standard_deviation: false,

            filter: Filter::new(),
            sort: Sort::Value,
            order: Order::Descending,
            priority: Priority::new(),

            view: View::Table,
            plot: PlotSettings::new(),
            // Reset
            reset_sum: false,
            reset_table: false,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.visuals_mut().collapsing_header_frame = true;

        self.precision(ui);
        self.significant(ui);
        self.sticky(ui);
        self.truncate(ui);

        self.mean_and_standard_deviation(ui);
        self.ddof(ui);

        // Filter
        ui.heading("Filter");
        ui.separator();

        // self.filter.show(ui, data_frame)?;

        // Sort
        ui.heading(ui.localize("Sort"));
        ui.separator();

        self.sort(ui);
        self.distance(ui);
        self.aggregation(ui);
        self.order(ui);

        ui.separator();

        // Plot
        ui.collapsing(
            RichText::from(ui.localize("PlotSettings")).heading(),
            |ui| {
                ui.add_enabled_ui(self.view == View::Plot, |ui| {
                    self.plot(ui);
                });
            },
        );
    }

    /// Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            Slider::new(&mut self.precision, 1..=MAX_PRECISION).ui(ui);
            if ui.button((BOOKMARK, "3")).clicked() {
                self.precision = 3;
            };
        });
    }

    /// Significant
    fn significant(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Significant"))
                .on_hover_localized("Significant.hover");
            ui.checkbox(&mut self.significant, ());
        });
    }

    /// Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            Slider::new(&mut self.sticky, 0..=NUM_COLUMNS).ui(ui);
        });
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("TruncateHeaders"))
                .on_hover_localized("TruncateHeaders.hover");
            ui.checkbox(&mut self.truncate, "");
        });
    }

    /// Mean and standard deviation
    fn mean_and_standard_deviation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Mean"))
                .on_hover_localized("Mean.hover");
            ui.checkbox(&mut self.mean, ());
            if !self.mean {
                self.standard_deviation = false;
                ui.disable();
            }
            ui.label(ui.localize("StandardDeviation"))
                .on_hover_localized("StandardDeviation.hover");
            ui.checkbox(&mut self.standard_deviation, ());
        });
    }

    /// DDOF
    /// https://numpy.org/devdocs/reference/generated/numpy.std.html
    fn ddof(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                .on_hover_localized("DeltaDegreesOfFreedom")
                .on_hover_ui(|ui| {
                    ui.hyperlink("https://numpy.org/devdocs/reference/generated/numpy.std.html");
                });
            Slider::new(&mut self.ddof, 0..=2).ui(ui);
        });
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByDistance"))
                .on_hover_localized("SortByDistance.hover");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(ui.localize(self.sort.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Key, ui.localize(Sort::Key.text()))
                        .on_hover_localized(Sort::Key.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Value,
                        ui.localize(Sort::Value.text()),
                    )
                    .on_hover_localized(Sort::Value.hover_text());
                })
                .response
                .on_hover_localized(self.sort.hover_text());
        });
    }

    /// Order
    fn order(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Order"));
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
        });
    }

    /// Distance
    fn distance(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByDistance"))
                .on_hover_localized("SortByDistance.hover");
            let enabled = self.sort == Sort::Value;
            ui.add_enabled_ui(enabled, |ui| {
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(ui.localize(self.priority.distance.text()))
                    .show_ui(ui, |ui| {
                        for distance in DISTANCES {
                            ui.selectable_value(
                                &mut self.priority.distance,
                                distance,
                                ui.localize(distance.text()),
                            )
                            .on_hover_localized(distance.hover_text());
                        }
                    })
                    .response
                    .on_hover_localized(self.priority.distance.hover_text());
            })
            .response
            .on_disabled_hover_text("Used only for sort by value");
        });
    }

    /// Aggregation
    fn aggregation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByAggregation"))
                .on_hover_localized("SortByAggregation.hover");
            let enabled = self.sort == Sort::Value;
            ui.add_enabled_ui(enabled, |ui| {
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(ui.localize(self.priority.aggregation.text()))
                    .show_ui(ui, |ui| {
                        for aggregation in AGGREGATIONS {
                            ui.selectable_value(
                                &mut self.priority.aggregation,
                                aggregation,
                                ui.localize(aggregation.text()),
                            )
                            .on_hover_localized(aggregation.hover_text());
                        }
                    })
                    .response
                    .on_hover_localized(self.priority.aggregation.hover_text());
            })
            .response
            .on_disabled_hover_text("Used only for sort by value");
        });
    }

    /// Plot
    fn plot(&mut self, ui: &mut Ui) {
        self.legend(ui);
        self.radius_of_points(ui);
        self.axis(ui);
    }

    /// Legend
    fn legend(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Legend"));
            ui.checkbox(&mut self.plot.legend, "");
        });
    }

    /// Radius of points
    fn radius_of_points(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
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
        });
    }

    /// Axis
    fn axis(&mut self, ui: &mut Ui) {
        for axis in [&mut self.plot.axes.x, &mut self.plot.axes.y] {
            ui.horizontal(|ui| {
                ui.label(ui.localize("PlotAxes"));
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(ui.localize(axis.text()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(axis, Axis::Alpha, ui.localize(Axis::Alpha.text()))
                            .on_hover_localized(Axis::Alpha.hover_text());
                        ui.selectable_value(
                            axis,
                            Axis::EquivalentChainLength,
                            ui.localize(Axis::EquivalentChainLength.text()),
                        )
                        .on_hover_localized(Axis::EquivalentChainLength.hover_text());
                        ui.selectable_value(
                            axis,
                            Axis::OnsetTemperature,
                            ui.localize(Axis::OnsetTemperature.text()),
                        )
                        .on_hover_localized(Axis::OnsetTemperature.hover_text());
                        ui.selectable_value(
                            axis,
                            Axis::TemperatureStep,
                            ui.localize(Axis::TemperatureStep.text()),
                        )
                        .on_hover_localized(Axis::TemperatureStep.hover_text());
                    })
                    .response
                    .on_hover_localized(axis.hover_text());
            });
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Key,
    Value,
}

impl Text for Sort {
    fn text(&self) -> &'static str {
        match self {
            Self::Key => "SortByKey",
            Self::Value => "SortByValue",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Key => "SortByKey.hover",
            Self::Value => "SortByValue.hover",
        }
    }
}

/// Priority
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Priority {
    pub(crate) aggregation: Aggregation,
    pub(crate) distance: Distance,
}

impl Priority {
    fn new() -> Self {
        Self {
            aggregation: Aggregation::Median,
            distance: Distance::Alpha,
        }
    }
}

/// Aggregation
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Aggregation {
    Maximum,
    Mean,
    Median,
    Minimum,
}

impl Aggregation {
    pub(crate) const fn id(&self) -> &'static str {
        match self {
            Self::Maximum => MAXIMUM,
            Self::Mean => MEAN,
            Self::Median => MEDIAN,
            Self::Minimum => MINIMUM,
        }
    }
}

impl Text for Aggregation {
    fn text(&self) -> &'static str {
        match self {
            Self::Maximum => "SortByMaximum",
            Self::Mean => "SortByMean",
            Self::Median => "SortByMedian",
            Self::Minimum => "SortByMinimum",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Maximum => "SortByMaximum.hover",
            Self::Mean => "SortByMean.hover",
            Self::Median => "SortByMedian.hover",
            Self::Minimum => "SortByMinimum.hover",
        }
    }
}

/// Distance
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Distance {
    Alpha,
    EquivalentChainLength,
    Euclidean,
}

impl Distance {
    pub(crate) const fn id(&self) -> &'static str {
        match self {
            Self::Alpha => ALPHA,
            Self::EquivalentChainLength => EQUIVALENT_CHAIN_LENGTH,
            Self::Euclidean => EUCLIDEAN,
        }
    }
}

impl Text for Distance {
    fn text(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::EquivalentChainLength => "EquivalentChainLength",
            Self::Euclidean => "Euclidean",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha.hover",
            Self::EquivalentChainLength => "EquivalentChainLength.hover",
            Self::Euclidean => "Euclidean.hover",
        }
    }
}

// /// Interpolation
// #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Interpolation {
//     pub(crate) onset_temperature: f64,
//     pub(crate) temperature_step: f64,
// }

// impl Interpolation {
//     pub fn new() -> Self {
//         Self {
//             onset_temperature: 0.0,
//             temperature_step: 0.0,
//         }
//     }
// }

// impl Hash for Interpolation {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.onset_temperature.ord().hash(state);
//         self.temperature_step.ord().hash(state);
//     }
// }

// /// Retention time settings
// #[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
// pub(crate) struct RetentionTime {
//     pub(crate) precision: usize,
//     pub(crate) units: TimeUnits,
// }

// impl RetentionTime {
//     pub(crate) fn format(self, value: f32) -> RetentionTimeFormat {
//         RetentionTimeFormat {
//             value,
//             precision: Some(self.precision),
//             units: self.units,
//         }
//     }
// }

// impl Default for RetentionTime {
//     fn default() -> Self {
//         Self {
//             precision: 2,
//             units: Default::default(),
//         }
//     }
// }

// #[derive(Clone, Copy, Debug, Default)]
// pub(crate) struct RetentionTimeFormat {
//     value: f32,
//     precision: Option<usize>,
//     units: TimeUnits,
// }

// impl RetentionTimeFormat {
//     pub(crate) fn precision(self, precision: Option<usize>) -> Self {
//         Self { precision, ..self }
//     }
// }

// impl Display for RetentionTimeFormat {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         let time = Time::new::<millisecond>(self.value as _);
//         let value = match self.units {
//             TimeUnits::Millisecond => time.get::<millisecond>(),
//             TimeUnits::Second => time.get::<second>(),
//             TimeUnits::Minute => time.get::<minute>(),
//         };
//         if let Some(precision) = self.precision {
//             write!(f, "{value:.precision$}")
//         } else {
//             write!(f, "{value}")
//         }
//     }
// }

// impl From<RetentionTimeFormat> for WidgetText {
//     fn from(value: RetentionTimeFormat) -> Self {
//         value.to_string().into()
//     }
// }

// /// Time units
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub enum TimeUnits {
//     Millisecond,
//     #[default]
//     Second,
//     Minute,
// }

// impl TimeUnits {
//     pub fn abbreviation(&self) -> &'static str {
//         Units::from(*self).abbreviation()
//     }

//     pub fn singular(&self) -> &'static str {
//         Units::from(*self).singular()
//     }

//     pub fn plural(&self) -> &'static str {
//         Units::from(*self).plural()
//     }
// }

// impl From<TimeUnits> for Units {
//     fn from(value: TimeUnits) -> Self {
//         match value {
//             TimeUnits::Millisecond => Units::millisecond(millisecond),
//             TimeUnits::Second => Units::second(second),
//             TimeUnits::Minute => Units::minute(minute),
//         }
//     }
// }
