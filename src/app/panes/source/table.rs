use crate::app::{
    panes::{MARGIN, widgets::float::FloatValue},
    states::source::{ID_SOURCE, State},
};
use egui::{Color32, Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui, WidgetText};
use egui_ext::ResponseExt;
use egui_l20n::{ResponseExt as _, UiExt};
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;

const NUM_COLUMNS: usize = top::DERIVATIVE.end;
const TOP: &[Range<usize>] = &[
    top::INDEX,
    top::MODE,
    top::FATTY_ACID,
    top::RETENTION_TIME,
    top::TEMPERATURE,
    top::CHAIN_LENGTH,
    top::MASS,
    top::DERIVATIVE,
];

/// Table view
#[derive(Debug)]
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(super) const fn new(data_frame: &'a DataFrame, state: &'a mut State) -> Self {
        Self { data_frame, state }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as _;
        let num_columns = NUM_COLUMNS;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.state.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.state.settings.sticky)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.state.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, top::INDEX) => {
                ui.heading(HASH).on_hover_localized("index");
            }
            (0, top::MODE) => {
                ui.heading(ui.localize("mode"))
                    .on_hover_localized("mode.hover");
            }
            (0, top::FATTY_ACID) => {
                ui.heading(ui.localize("fatty-acid"))
                    .on_hover_localized("fatty-acid.abbreviation");
            }
            (0, top::RETENTION_TIME) => {
                ui.heading(ui.localize("retention-time"))
                    .on_hover_localized("retention-time.abbreviation")
                    .on_hover_localized("retention-time.hover");
            }
            (0, top::TEMPERATURE) => {
                ui.heading(ui.localize("temperature"))
                    .on_hover_localized("temperature.abbreviation")
                    .on_hover_localized("temperature.hover");
            }
            (0, top::CHAIN_LENGTH) => {
                ui.heading(ui.localize("chain-length"))
                    .on_hover_localized("chain-length.hover");
            }
            (0, top::MASS) => {
                ui.heading(ui.localize("mass"))
                    .on_hover_localized("mass.hover");
            }
            (0, top::DERIVATIVE) => {
                ui.heading(ui.localize("derivative"))
                    .on_hover_localized("derivative.hover");
            }
            // Bottom
            (1, bottom::ONSET) => {
                ui.heading(ui.localize("onset-temperature.abbreviation"))
                    .on_hover_localized("onset-temperature");
            }
            (1, bottom::STEP) => {
                ui.heading(ui.localize("temperature-step.abbreviation"))
                    .on_hover_localized("temperature-step")
                    .on_hover_localized("temperature-step.hover");
            }
            (1, bottom::ABSOLUTE) => {
                ui.heading(ui.localize("absolute-retention-time"))
                    .on_hover_localized("absolute-retention-time.hover");
            }
            (1, bottom::RELATIVE) => {
                ui.heading(ui.localize("relative-retention-time"))
                    .on_hover_localized("relative-retention-time.hover");
            }
            (1, bottom::DELTA) => {
                ui.heading(ui.localize("delta-retention-time"))
                    .on_hover_localized("delta-retention-time.hover");
            }
            (1, bottom::ECL) => {
                ui.heading(ui.localize("equivalent-chain-length.abbreviation"))
                    .on_hover_localized("equivalent-chain-length");
            }
            (1, bottom::FCL) => {
                ui.heading(ui.localize("fractional-chain-length.abbreviation"))
                    .on_hover_localized("fractional-chain-length");
            }
            (1, bottom::ECN) => {
                ui.heading(ui.localize("equivalent-carbon-number.abbreviation"))
                    .on_hover_localized("equivalent-carbon-number");
            }
            (1, bottom::SLOPE) => {
                ui.heading(ui.localize("slope"));
            }
            (1, bottom::ANGLE) => {
                ui.heading(ui.localize("angle"));
            }
            _ => {}
        }
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
        match (row, column) {
            (row, top::INDEX) => {
                ui.label(row.to_string());
            }
            (row, bottom::ONSET) => {
                let mode = self.data_frame["Mode"].struct_()?;
                let onset_temperature = mode.field_by_name("OnsetTemperature")?;
                ui.label(onset_temperature.str_value(row)?)
                    .on_hover_ui(|ui| {
                        (|| -> PolarsResult<()> {
                            let Some(dead_time) = self.data_frame["DeadTime"].f64()?.get(row)
                            else {
                                polars_bail!(NoData: "DeadTime[{row}]");
                            };
                            ui.label(dead_time.to_string());
                            Ok(())
                        })()
                        .unwrap()
                    });
            }
            (row, bottom::STEP) => {
                let mode = self.data_frame["Mode"].struct_()?;
                let temperature_step = mode.field_by_name("TemperatureStep")?;
                ui.label(temperature_step.str_value(row)?);
            }
            (row, top::FATTY_ACID) => {
                ui.label(self.data_frame[FATTY_ACID].str()?.get(row).unwrap_or("-"));
            }
            (row, bottom::ABSOLUTE) => {
                let absolute_series = self.data_frame["RetentionTime"]
                    .struct_()?
                    .field_by_name("Absolute")?;
                let mean_series = absolute_series.struct_()?.field_by_name("Mean")?;
                let standard_deviation_series = absolute_series
                    .struct_()?
                    .field_by_name("StandardDeviation")?;
                if let Some(standard_deviation) = standard_deviation_series.f64()?.get(row) {
                    if standard_deviation > 0.1 {
                        ui.visuals_mut().override_text_color = Some(Color32::RED);
                    } else if standard_deviation > 0.05 {
                        ui.visuals_mut().override_text_color = Some(Color32::YELLOW);
                    }
                }
                let text = mean_series
                    .f64()?
                    .get(row)
                    .map_or(WidgetText::from("-"), |mean| {
                        WidgetText::from(format!("{mean}"))
                    });
                ui.label(text)
                    .try_on_hover_ui(|ui| {
                        let Some(mean) = mean_series.f64()?.get(row) else {
                            polars_bail!(NoData: "Mean[{row}]");
                        };
                        let Some(standard_deviation) = standard_deviation_series.f64()?.get(row)
                        else {
                            polars_bail!(NoData: "StandardDeviation[{row}]");
                        };
                        ui.label(format!("{mean} ±{standard_deviation}"));
                        Ok(())
                    })?
                    .try_on_hover_ui(|ui| {
                        ui.heading("Sample");
                        let Some(sample) = absolute_series
                            .struct_()?
                            .field_by_name("Sample")?
                            .list()?
                            .get_as_series(row)
                        else {
                            polars_bail!(NoData: "Sample[{row}]");
                        };
                        ui.vertical(|ui| {
                            for value in sample.iter() {
                                ui.label(value.to_string());
                            }
                        });
                        Ok(())
                    })?;
            }
            (row, bottom::RELATIVE) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let relative_series = retention_time.field_by_name("Relative")?;
                ui.add(
                    FloatValue::new(relative_series.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, bottom::DELTA) => {
                let retention_time = self.data_frame["RetentionTime"].struct_()?;
                let delta = retention_time.field_by_name("Delta")?;
                ui.add(
                    FloatValue::new(delta.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, top::TEMPERATURE) => {
                let temperature = &self.data_frame["Temperature"];
                ui.add(
                    FloatValue::new(temperature.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, bottom::ECL) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let ecl = chain_length.field_by_name("EquivalentChainLength")?;
                ui.add(
                    FloatValue::new(ecl.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, bottom::FCL) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let fcl = chain_length.field_by_name("FCL")?;
                ui.add(
                    FloatValue::new(fcl.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, bottom::ECN) => {
                let chain_length = self.data_frame["ChainLength"].struct_()?;
                let ecn = chain_length.field_by_name("ECN")?;
                ui.label(ecn.str_value(row)?);
            }
            (row, top::MASS) => {
                let mass = self.data_frame["Mass"].struct_()?;
                let rcooch3 = mass.field_by_name("RCOOCH3")?;
                ui.add(
                    FloatValue::new(rcooch3.f64()?.get(row))
                        .precision(Some(self.state.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        (|| -> PolarsResult<()> {
                            ui.label("[RCO]+");
                            let rcoo = mass.field_by_name("RCO")?;
                            ui.label(rcoo.str_value(row)?);
                            ui.end_row();

                            ui.label("[RCOO]-");
                            let rcoo = mass.field_by_name("RCOO")?;
                            ui.label(rcoo.str_value(row)?);
                            ui.end_row();

                            ui.label("RCOOH");
                            let rcooh = mass.field_by_name("RCOOH")?;
                            ui.label(rcooh.str_value(row)?);
                            ui.end_row();

                            ui.label("RCOOCH3");
                            ui.label(rcooch3.str_value(row)?);

                            Ok(())
                        })()
                        .unwrap()
                    });
                });
            }
            (row, bottom::SLOPE) => {
                let derivative = self.data_frame["Derivative"].struct_()?;
                let slope = derivative.field_by_name("Slope")?;
                ui.add(
                    FloatValue::new(slope.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            (row, bottom::ANGLE) => {
                let derivative = self.data_frame["Derivative"].struct_()?;
                let angle = derivative.field_by_name("Angle")?;
                ui.add(
                    FloatValue::new(angle.f64()?.get(row))
                        .precision(Some(self.state.settings.precision))
                        .hover(),
                );
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr.is_multiple_of(2) {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                    .unwrap()
            });
    }
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(super) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 1;
    pub(super) const RETENTION_TIME: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 3;
    pub(super) const TEMPERATURE: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(super) const CHAIN_LENGTH: Range<usize> = TEMPERATURE.end..TEMPERATURE.end + 3;
    pub(super) const MASS: Range<usize> = CHAIN_LENGTH.end..CHAIN_LENGTH.end + 1;
    pub(super) const DERIVATIVE: Range<usize> = MASS.end..MASS.end + 2;
}

mod bottom {
    use super::*;

    // Mode
    pub(super) const ONSET: Range<usize> = top::MODE.start..top::MODE.start + 1;
    pub(super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
    // Retention time
    pub(super) const ABSOLUTE: Range<usize> =
        top::RETENTION_TIME.start..top::RETENTION_TIME.start + 1;
    pub(super) const RELATIVE: Range<usize> = ABSOLUTE.end..ABSOLUTE.end + 1;
    pub(super) const DELTA: Range<usize> = RELATIVE.end..RELATIVE.end + 1;
    // Chain length
    pub(super) const ECL: Range<usize> = top::CHAIN_LENGTH.start..top::CHAIN_LENGTH.start + 1;
    pub(super) const FCL: Range<usize> = ECL.end..ECL.end + 1;
    pub(super) const ECN: Range<usize> = FCL.end..FCL.end + 1;
    // Derivative
    pub(super) const SLOPE: Range<usize> = top::DERIVATIVE.start..top::DERIVATIVE.start + 1;
    pub(super) const ANGLE: Range<usize> = SLOPE.end..SLOPE.end + 1;
}
