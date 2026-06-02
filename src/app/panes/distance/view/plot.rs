use crate::app::{
    computers::{distance::view::plot::Value, plot::IndexKey},
    states::distance::Settings,
};
use egui::Ui;
use egui_ext::color;
use egui_l10n::prelude::*;
use egui_plot::{AxisHints, Legend, Line, Plot, PlotPoint, PlotPoints, Points, VPlacement};
use polars::prelude::*;
use std::fmt::Write;
use tracing::error;

/// Plot view
#[derive(Clone)]
pub(crate) struct PlotView<'a> {
    pub(crate) value: Value,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(value: Value, settings: &'a Settings) -> Self {
        Self { value, settings }
    }
}

impl PlotView<'_> {
    pub(crate) fn show(self, ui: &mut Ui) {
        if let Err(error) = self.try_show(ui) {
            error!(%error);
        }
    }

    fn try_show(self, ui: &mut Ui) -> PolarsResult<()> {
        let mut plot = Plot::new("plot")
            // .allow_drag(context.settings.visualization.drag)
            // .allow_scroll(context.settings.visualization.scroll)
            ;
        if self.settings.plot.control.legend {
            plot = plot.legend(Legend::default().follow_insertion_order(true));
        }
        let onset_temperature = ui.localize("OnsetTemperature");
        let temperature_step = ui.localize("TemperatureStep");
        let alpha = ui.localize("Alpha");
        let index = self.value.index.clone();
        plot = plot
            // .x_axis_label(&temperature_step)
            .custom_x_axes(vec![
                AxisHints::new_x()
                    .label(&temperature_step)
                    .placement(VPlacement::Top),
                AxisHints::new_x().label(&onset_temperature),
                AxisHints::new_x().label("X"),
            ])
            .y_axis_label(&alpha)
            .label_formatter(move |name, &PlotPoint { x, y }| {
                if !name.is_empty() {
                    let mut label = String::new();
                    _ = write!(&mut label, "{name}");
                    if let Some(index) = index.get(&IndexKey(PlotPoint::new(x, y))) {
                        for (key, value) in index {
                            _ = writeln!(&mut label);
                            _ = write!(&mut label, "{key} = {value:?}");
                        }
                    }
                    label
                } else {
                    format!("{onset_temperature} = {x}\n{alpha} = {y}")
                }
            });
        plot.show(ui, |ui| -> PolarsResult<()> {
            for ((rank, onset_temperature), points) in &self.value.onset_temperature {
                // Line
                // let first = std::cmp::min(data.fatty_acids[0], data.fatty_acids[1]);
                let [from, to] = &self.value.fatty_acids[rank];
                let name = format!("{:#}-{:#}", from.delta(), to.delta(),);
                let line = Line::new(&name, PlotPoints::Borrowed(points))
                    .color(color(onset_temperature.0 as _));
                ui.line(line);
                // Points
                let points = Points::new(name, PlotPoints::Borrowed(points))
                    .color(color(*rank as _))
                    .radius(self.settings.plot.control.radius_of_points);
                ui.points(points);
            }
            // for ((temperature_step, rank), points) in &self.value.temperature_step {
            //     // Line
            //     // let first = std::cmp::min(data.fatty_acids[0], data.fatty_acids[1]);
            //     let [from, to] = &self.value.fatty_acids[rank];
            //     let name = format!("{:#}-{:#}", from.display(COMMON), to.display(COMMON),);
            //     let line = Line::new(PlotPoints::Borrowed(points))
            //         .name(&name)
            //         .color(color(*rank as _));
            //     ui.line(line);
            //     // Points
            //     // let points = Points::new(PlotPoints::Borrowed(points))
            //     //     .name(name)
            //     //     .color(color(rank as _))
            //     //     .radius(self.settings.radius_of_points);
            //     // ui.points(points);
            // }
            Ok(())
        });
        Ok(())
    }
}
