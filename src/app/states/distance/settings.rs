use crate::{
    app::{
        panes::distance::view::table::NUM_COLUMNS,
        states::source::settings::{Filter, Plot, View},
    },
    r#const::{EQUIVALENT_CHAIN_LENGTH, MAXIMUM, MEAN, MEDIAN, MINIMUM, SELECTIVITY_FACTOR},
    localization::Text,
};
use const_format::formatcp;
use egui::{ComboBox, RichText, Slider, Ui, Widget as _};
use egui_l10n::prelude::ContextExt;
use egui_phosphor::regular::BOOKMARK;
use serde::{Deserialize, Serialize};
use widgets::settings::{MeanAndStandardDeviation, Order, Precision};

const AGGREGATIONS: [Aggregation; 4] = [
    Aggregation::Maximum,
    Aggregation::Mean,
    Aggregation::Median,
    Aggregation::Minimum,
];

const DISTANCES: [Distance; 2] = [Distance::EquivalentChainLength, Distance::SelectivityFactor];

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: Precision,
    pub(crate) mean_and_standard_deviation: MeanAndStandardDeviation,

    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) filter: Filter,
    pub(crate) sort: Sort,
    pub(crate) order: Order,
    pub(crate) priority: Priority,

    pub(crate) view: View,
    pub(crate) plot: Plot,

    // Reset
    pub(crate) reset: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            precision: Precision::new(),
            mean_and_standard_deviation: MeanAndStandardDeviation::new(),

            resizable: false,
            sticky: 0,
            truncate: false,

            filter: Filter::new(),
            sort: Sort::Value,
            order: Order::new(),
            priority: Priority::new(),

            view: View::Table,
            plot: Plot::new(),

            // Reset
            reset: false,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.visuals_mut().collapsing_header_frame = true;

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            self.precision.show(ui);
        });

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            self.mean_and_standard_deviation.show(ui);
        });

        self.sticky(ui);
        self.truncate(ui);

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
        self.order.show(ui);

        ui.separator();

        // Plot
        ui.collapsing(
            RichText::from(ui.localize("PlotSettings")).heading(),
            |ui| {
                ui.add_enabled_ui(self.view == View::Plot, |ui| {
                    self.plot.show(ui);
                });
            },
        );
    }

    /// Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
                ui.label(ui.localize("StickyColumns.hover"));
            });
            Slider::new(&mut self.sticky, 0..=NUM_COLUMNS).ui(ui);
        });
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
                ui.label(ui.localize("TruncateHeaders.hover"));
            });
            ui.checkbox(&mut self.truncate, "");
        });
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByDistance")).on_hover_ui(|ui| {
                ui.label(ui.localize("SortByDistance.hover"));
            });
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(ui.localize(self.sort.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Key, ui.localize(Sort::Key.text()))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(Sort::Key.hover_text()));
                        });
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Value,
                        ui.localize(Sort::Value.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Sort::Value.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.sort.hover_text()));
                });
        });
    }

    /// Distance
    fn distance(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByDistance")).on_hover_ui(|ui| {
                ui.label(ui.localize("SortByDistance.hover"));
            });
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
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize(distance.hover_text()));
                            });
                        }
                    })
                    .response
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(self.priority.distance.hover_text()));
                    });
            })
            .response
            .on_disabled_hover_text("Used only for sort by value");
        });
    }

    /// Aggregation
    fn aggregation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SortByAggregation"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("SortByAggregation.hover"));
                });
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
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize(aggregation.hover_text()));
                            });
                        }
                    })
                    .response
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(self.priority.aggregation.hover_text()));
                    });
            })
            .response
            .on_disabled_hover_text("Used only for sort by value");
        });
    }

    // /// Axis
    // fn axis(&mut self, ui: &mut Ui) {
    //     for axis in [&mut self.plot.axes.x, &mut self.plot.axes.y] {
    //         ui.horizontal(|ui| {
    //             ui.label(ui.localize("PlotAxes"));
    //             ComboBox::from_id_salt(ui.next_auto_id())
    //                 .selected_text(ui.localize(axis.text()))
    //                 .show_ui(ui, |ui| {
    //                     ui.selectable_value(
    //                         axis,
    //                         Axis::SelectivityFactor,
    //                         ui.localize(Axis::SelectivityFactor.text()),
    //                     )
    //                     .on_hover_ui(|ui| {
    //                         ui.label(ui.localize(Axis::SelectivityFactor.hover_text()));
    //                     });
    //                     ui.selectable_value(
    //                         axis,
    //                         Axis::EquivalentChainLength,
    //                         ui.localize(Axis::EquivalentChainLength.text()),
    //                     )
    //                     .on_hover_ui(|ui| {
    //                         ui.label(ui.localize(Axis::EquivalentChainLength.hover_text()));
    //                     });
    //                     ui.selectable_value(
    //                         axis,
    //                         Axis::OnsetTemperature,
    //                         ui.localize(Axis::OnsetTemperature.text()),
    //                     )
    //                     .on_hover_ui(|ui| {
    //                         ui.label(ui.localize(Axis::OnsetTemperature.hover_text()));
    //                     });
    //                     ui.selectable_value(
    //                         axis,
    //                         Axis::TemperatureStep,
    //                         ui.localize(Axis::TemperatureStep.text()),
    //                     )
    //                     .on_hover_ui(|ui| {
    //                         ui.label(ui.localize(Axis::TemperatureStep.hover_text()));
    //                     });
    //                 })
    //                 .response
    //                 .on_hover_ui(|ui| {
    //                     ui.label(ui.localize(axis.hover_text()));
    //                 });
    //         });
    //     }
    // }
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
            distance: Distance::SelectivityFactor,
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
    EquivalentChainLength,
    SelectivityFactor,
}

impl Distance {
    pub(crate) const fn id(&self) -> &'static str {
        match self {
            Self::EquivalentChainLength => EQUIVALENT_CHAIN_LENGTH,
            Self::SelectivityFactor => SELECTIVITY_FACTOR,
        }
    }
}

impl Text for Distance {
    fn text(&self) -> &'static str {
        match self {
            Self::EquivalentChainLength => EQUIVALENT_CHAIN_LENGTH,
            Self::SelectivityFactor => SELECTIVITY_FACTOR,
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::EquivalentChainLength => formatcp!("{EQUIVALENT_CHAIN_LENGTH}.hover"),
            Self::SelectivityFactor => formatcp!("{SELECTIVITY_FACTOR}.hover"),
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
