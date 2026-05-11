use crate::{
    app::{
        panes::MARGIN,
        states::source::{ID_SOURCE, Settings},
        widgets::array::Float64Array,
    },
    r#const::{
        ABSOLUTE, ADJUSTED, ARRAY, BACKWARD, CHAIN_LENGTH, DEAD_TIME, EQUIVALENT_CARBON_NUMBER,
        EQUIVALENT_CHAIN_LENGTH, FORWARD, FRACTIONAL_CHAIN_LENGTH, MASS, MODE, ONSET_TEMPERATURE,
        RELATIVE, RETENTION_FACTOR, RETENTION_TIME, STANDARD, TEMPERATURE, TEMPERATURE_STEP,
    },
    utils::polars::SeriesExt as _,
};
use const_format::formatcp;
use egui::{Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_ext::prelude::*;
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::option::DisplayOption;
use std::{f64, iter::zip, ops::Range};
use tracing::instrument;

pub(crate) const NUM_COLUMNS: usize = top::MASS.end;

const TOP: &[Range<usize>] = &[
    top::INDEX,
    top::MODE,
    top::FATTY_ACID,
    top::RETENTION_TIME,
    top::RETENTION_FACTOR,
    top::DEAD_TIME,
    top::TEMPERATURE,
    top::CHAIN_LENGTH,
    top::MASS,
];

/// Table view
#[derive(Debug)]
pub(crate) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a mut Settings,
}

impl<'a> TableView<'a> {
    pub(crate) const fn new(data_frame: &'a DataFrame, settings: &'a mut Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }
}

impl TableView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.settings.reset {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.settings.reset = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as _;
        let num_columns = NUM_COLUMNS;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
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
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, top::INDEX) => {
                ui.heading(HASH).on_hover_localized(INDEX);
            }
            (0, top::MODE) => {
                ui.heading(ui.localize(MODE))
                    .on_hover_localized(formatcp!("{MODE}.hover"));
            }
            (0, top::FATTY_ACID) => {
                ui.heading(ui.localize(FATTY_ACID))
                    .on_hover_localized(formatcp!("{FATTY_ACID}.abbreviation"));
            }
            (0, top::RETENTION_TIME) => {
                ui.heading(ui.localize(RETENTION_TIME))
                    .on_hover_localized(formatcp!("{RETENTION_TIME}.abbreviation"))
                    .on_hover_localized(formatcp!("{RETENTION_TIME}.hover"));
            }
            (0, top::RETENTION_FACTOR) => {
                ui.heading(ui.localize(RETENTION_FACTOR))
                    .on_hover_localized(formatcp!("{RETENTION_FACTOR}.hover"))
                    .on_hover_ui(|ui| {
                        ui.markdown(include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/doc/en/RetentionFactor.md"
                        )));
                    });
            }
            (0, top::DEAD_TIME) => {
                ui.heading(ui.localize(DEAD_TIME))
                    .on_hover_localized(formatcp!("{DEAD_TIME}.abbreviation"))
                    .on_hover_localized(formatcp!("{DEAD_TIME}.hover"));
            }
            (0, top::CHAIN_LENGTH) => {
                ui.heading(ui.localize("ChainLength"))
                    .on_hover_localized("ChainLength.hover");
            }
            (0, top::TEMPERATURE) => {
                ui.heading(ui.localize(TEMPERATURE))
                    .on_hover_localized(formatcp!("{TEMPERATURE}.abbreviation"))
                    .on_hover_localized(formatcp!("{TEMPERATURE}.hover"));
            }
            (0, top::MASS) => {
                ui.heading(ui.localize(MASS))
                    .on_hover_localized(formatcp!("{MASS}.hover"));
            }
            // Bottom
            (1, bottom::ONSET) => {
                ui.heading(ui.localize(formatcp!("{ONSET_TEMPERATURE}.abbreviation")))
                    .on_hover_localized(ONSET_TEMPERATURE);
            }
            (1, bottom::STEP) => {
                ui.heading(ui.localize(formatcp!("{TEMPERATURE_STEP}.abbreviation")))
                    .on_hover_localized(TEMPERATURE_STEP)
                    .on_hover_localized(formatcp!("{TEMPERATURE_STEP}.hover"));
            }
            (1, bottom::ABSOLUTE) => {
                ui.heading(ui.localize(formatcp!("{ABSOLUTE}{RETENTION_TIME}")))
                    .on_hover_localized(formatcp!("{ABSOLUTE}{RETENTION_TIME}.hover"));
            }
            (1, bottom::RELATIVE) => {
                ui.heading(ui.localize(formatcp!("{RELATIVE}{RETENTION_TIME}")))
                    .on_hover_localized(formatcp!("{RELATIVE}{RETENTION_TIME}.hover"));
            }
            (1, bottom::ADJUSTED) => {
                ui.heading(ui.localize(formatcp!("{ADJUSTED}{RETENTION_TIME}")))
                    .on_hover_localized(formatcp!("{ADJUSTED}{RETENTION_TIME}.hover"));
            }
            (1, bottom::ECL) => {
                ui.heading(ui.localize(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.abbreviation")))
                    .on_hover_localized(EQUIVALENT_CHAIN_LENGTH)
                    .on_hover_ui(|ui| {
                        ui.markdown(include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/doc/en/EquivalentChainLength.md"
                        )));
                    });
            }
            (1, bottom::FCL) => {
                ui.heading(ui.localize(formatcp!("{FRACTIONAL_CHAIN_LENGTH}.abbreviation")))
                    .on_hover_localized(FRACTIONAL_CHAIN_LENGTH)
                    .on_hover_ui(|ui| {
                        ui.markdown(include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/doc/en/FractionalChainLength.md"
                        )));
                    });
            }
            (1, bottom::ECN) => {
                ui.heading(ui.localize(formatcp!("{EQUIVALENT_CARBON_NUMBER}.abbreviation")))
                    .on_hover_localized(EQUIVALENT_CARBON_NUMBER);
            }
            _ => {}
        }
    }

    #[instrument(skip(self, ui), err)]
    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, top::INDEX) => {
                ui.label(row.to_string());
            }
            (row, bottom::ONSET) => {
                ui.label(
                    self.data_frame[MODE]
                        .struct_()?
                        .field_by_name(ONSET_TEMPERATURE)?
                        .str_value(row)?,
                );
            }
            (row, bottom::STEP) => {
                ui.label(
                    self.data_frame[MODE]
                        .struct_()?
                        .field_by_name(TEMPERATURE_STEP)?
                        .str_value(row)?,
                );
            }
            (row, top::FATTY_ACID) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .as_materialized_series()
                        .str_value(row)?,
                );
            }
            (row, bottom::ABSOLUTE) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[RETENTION_TIME]
                            .struct_()?
                            .field_by_name(ABSOLUTE)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::RELATIVE) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[RETENTION_TIME]
                            .struct_()?
                            .field_by_name(RELATIVE)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                        for (retention_time, standard_retention_time) in
                            zip(&self.retention_times(row)?, &self._retention_time(row)?)
                        {
                            let retention_time = retention_time.display();
                            let standard_retention_time = standard_retention_time.display();
                            ui.label(format!("{retention_time:#} / {standard_retention_time:#}"));
                        }
                        Ok(())
                    })?;
            }
            (row, bottom::ADJUSTED) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[RETENTION_TIME]
                            .struct_()?
                            .field_by_name(ADJUSTED)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                        let dead_time = self.dead_time(row)?;
                        for retention_time in &self.retention_times(row)? {
                            let retention_time = retention_time.display();
                            ui.label(format!("{retention_time:#} - {dead_time}"));
                        }
                        Ok(())
                    })?;
            }
            (row, top::RETENTION_FACTOR) => {
                Float64Array::builder()
                    .series(self.data_frame[RETENTION_FACTOR].as_materialized_series())
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                        let Some(adjusted_retention_times) = self.data_frame[RETENTION_TIME]
                            .struct_()?
                            .field_by_name(ADJUSTED)?
                            .struct_()?
                            .field_by_name(ARRAY)?
                            .array()?
                            .get_as_series(row)
                        else {
                            return Err(
                                polars_err!(NoData: "{RETENTION_TIME}.{ADJUSTED}.{ARRAY}[{row}]"),
                            );
                        };
                        let dead_time = self.dead_time(row)?;
                        for adjusted_retention_time in adjusted_retention_times.f64()? {
                            let retention_time = adjusted_retention_time.display();
                            ui.label(format!("{retention_time:#} / {dead_time}"));
                        }
                        Ok(())
                    })?;
            }
            (row, top::DEAD_TIME) => {
                let dead_time = self.dead_time(row)?;
                ui.label(dead_time.to_string());
            }
            (row, bottom::ECL) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(EQUIVALENT_CHAIN_LENGTH)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                        let (
                            forward_carbons,
                            backward_carbons,
                            forward_retention_times,
                            backward_retention_times,
                        ) = self._equivalent_chain_length(row)?;
                        for (retention_time, ((forward_carbon, backward_carbon), (forward_retention_time, backward_retention_time))) in
                            zip(&self.retention_times(row)?, zip(&forward_carbons, &backward_carbons).zip(zip(&forward_retention_times, &backward_retention_times)))
                        {
                            let forward_carbon = forward_carbon.display();
                            let backward_carbon = backward_carbon.display();
                            let retention_time = retention_time.display();
                            let forward_retention_time = forward_retention_time.display();
                            let backward_retention_time = backward_retention_time.display();
                            ui.label(format!("{forward_carbon:#} + ({backward_carbon:#} - {forward_carbon:#}) * ({retention_time:#} - {forward_retention_time:#}) / ({backward_retention_time:#} - {forward_retention_time:#})"));
                        }
                        Ok(())
                    })?;
            }
            (row, bottom::FCL) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(FRACTIONAL_CHAIN_LENGTH)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                        let carbon = self._fractional_chain_length(row)?;
                        for equivalent_chain_length in &self.equivalent_chain_lengths(row)? {
                            let equivalent_chain_length = equivalent_chain_length.display();
                            ui.label(format!("{equivalent_chain_length:#} - {carbon}"));
                        }
                        Ok(())
                    })?;
            }
            (row, bottom::ECN) => {
                let ecn_series = self.data_frame[CHAIN_LENGTH]
                    .struct_()?
                    .field_by_name(EQUIVALENT_CARBON_NUMBER)?;
                let text = ecn_series.str_value(row)?;
                ui.label(text);
            }
            (row, top::TEMPERATURE) => {
                Float64Array::builder()
                    .series(self.data_frame[TEMPERATURE].as_materialized_series())
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        let onset_temperature = self.onset_temperature(row)?;
                        let temperature_step = self.temperature_step(row)?;
                        for retention_time in &self.retention_times(row)? {
                            let retention_time = retention_time.display();
                            ui.label(format!("max({onset_temperature} + {retention_time:#} * {temperature_step}; 250)"));
                        }
                        Ok(())
                    })?;
            }
            (row, top::MASS) => {
                let mass = self.data_frame[MASS].struct_()?;
                let rcooch3 = mass.field_by_name("RCOOCH3")?;
                ui.label(rcooch3.str_f64(row)?).try_on_hover_ui(|ui| {
                    Grid::new(ui.next_auto_id())
                        .show(ui, |ui| -> PolarsResult<()> {
                            ui.label("[RCO]+");
                            ui.label(mass.field_by_name("RCO")?.str_f64(row)?);
                            ui.end_row();

                            ui.label("[RCOO]-");
                            ui.label(mass.field_by_name("RCOO")?.str_f64(row)?);
                            ui.end_row();

                            ui.label("RCOOH");
                            ui.label(mass.field_by_name("RCOOH")?.str_f64(row)?);
                            ui.end_row();

                            ui.label("RCOOCH3");
                            ui.label(rcooch3.str_f64(row)?);
                            ui.end_row();
                            Ok(())
                        })
                        .inner
                })?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn onset_temperature(&self, row: usize) -> PolarsResult<f64> {
        let Some(onset_temperature) = self.data_frame[MODE]
            .struct_()?
            .field_by_name(ONSET_TEMPERATURE)?
            .f64()?
            .get(row)
        else {
            return Err(polars_err!(NoData: "{MODE}.{ONSET_TEMPERATURE}[{row}]"));
        };
        Ok(onset_temperature)
    }

    fn dead_time(&self, row: usize) -> PolarsResult<f64> {
        let Some(dead_time) = self.data_frame[DEAD_TIME].f64()?.get(row) else {
            return Err(polars_err!(NoData: "{DEAD_TIME}[{row}]"));
        };
        Ok(dead_time)
    }

    fn retention_times(&self, row: usize) -> PolarsResult<Float64Chunked> {
        let Some(retention_time) = self.data_frame[RETENTION_TIME]
            .struct_()?
            .field_by_name(ABSOLUTE)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{RETENTION_TIME}.{ABSOLUTE}.{ARRAY}[{row}]"));
        };
        Ok(retention_time.f64()?.clone())
    }

    fn temperature_step(&self, row: usize) -> PolarsResult<f64> {
        let Some(temperature_step) = self.data_frame[MODE]
            .struct_()?
            .field_by_name(TEMPERATURE_STEP)?
            .f64()?
            .get(row)
        else {
            return Err(polars_err!(NoData: "{MODE}.{TEMPERATURE_STEP}[{row}]"));
        };
        Ok(temperature_step)
    }

    fn equivalent_chain_lengths(&self, row: usize) -> PolarsResult<Float64Chunked> {
        let Some(equivalent_chain_length) = self.data_frame[CHAIN_LENGTH]
            .struct_()?
            .field_by_name(EQUIVALENT_CHAIN_LENGTH)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "_._{CHAIN_LENGTH}{EQUIVALENT_CHAIN_LENGTH}[{row}]"));
        };
        Ok(equivalent_chain_length.f64()?.clone())
    }

    fn _retention_time(&self, row: usize) -> PolarsResult<Float64Chunked> {
        let Some(standard) = self.data_frame["_"]
            .struct_()?
            .field_by_name(formatcp!("_{RETENTION_TIME}"))?
            .struct_()?
            .field_by_name(STANDARD)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "_{RETENTION_TIME}.{STANDARD}[{row}]"));
        };
        Ok(standard.f64()?.clone())
    }

    fn _equivalent_chain_length(
        &self,
        row: usize,
    ) -> PolarsResult<(UInt8Chunked, UInt8Chunked, Float64Chunked, Float64Chunked)> {
        let equivalent_chain_length = self.data_frame["_"]
            .struct_()?
            .field_by_name(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))?
            .struct_()?
            .clone();
        let Some(forward_carbon) = equivalent_chain_length
            .field_by_name(formatcp!("{FORWARD}{CARBON}"))?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "_{EQUIVALENT_CHAIN_LENGTH}.{FORWARD}{CARBON}[{row}]"));
        };
        let Some(backward_carbon) = equivalent_chain_length
            .field_by_name(formatcp!("{BACKWARD}{CARBON}"))?
            .array()?
            .get_as_series(row)
        else {
            return Err(
                polars_err!(NoData: "_{EQUIVALENT_CHAIN_LENGTH}.{BACKWARD}{CARBON}[{row}]"),
            );
        };
        let Some(forward_retention_times) = equivalent_chain_length
            .field_by_name(formatcp!("{FORWARD}{RETENTION_TIME}"))?
            .array()?
            .get_as_series(row)
        else {
            return Err(
                polars_err!(NoData: "_{EQUIVALENT_CHAIN_LENGTH}.{FORWARD}{RETENTION_TIME}[{row}]"),
            );
        };
        let Some(backward_retention_times) = equivalent_chain_length
            .field_by_name(formatcp!("{BACKWARD}{RETENTION_TIME}"))?
            .array()?
            .get_as_series(row)
        else {
            return Err(
                polars_err!(NoData: "_{EQUIVALENT_CHAIN_LENGTH}.{BACKWARD}{RETENTION_TIME}[{row}]"),
            );
        };
        Ok((
            forward_carbon.u8()?.clone(),
            backward_carbon.u8()?.clone(),
            forward_retention_times.f64()?.clone(),
            backward_retention_times.f64()?.clone(),
        ))
    }

    fn _fractional_chain_length(&self, row: usize) -> PolarsResult<u8> {
        let Some(carbon) = self.data_frame["_"]
            .struct_()?
            .field_by_name(formatcp!("_{FRACTIONAL_CHAIN_LENGTH}"))?
            .struct_()?
            .field_by_name(CARBON)?
            .u8()?
            .get(row)
        else {
            return Err(polars_err!(NoData: "_{FRACTIONAL_CHAIN_LENGTH}.{CARBON}[{row}]"));
        };
        Ok(carbon)
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
                _ = self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

mod top {
    use super::*;

    pub(crate) const INDEX: Range<usize> = 0..1;
    pub(crate) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(crate) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 1;
    pub(crate) const RETENTION_TIME: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 3;
    pub(crate) const RETENTION_FACTOR: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(crate) const DEAD_TIME: Range<usize> = RETENTION_FACTOR.end..RETENTION_FACTOR.end + 1;
    pub(crate) const CHAIN_LENGTH: Range<usize> = DEAD_TIME.end..DEAD_TIME.end + 3;
    pub(crate) const TEMPERATURE: Range<usize> = CHAIN_LENGTH.end..CHAIN_LENGTH.end + 1;
    pub(crate) const MASS: Range<usize> = TEMPERATURE.end..TEMPERATURE.end + 1;
}

mod bottom {
    use super::*;

    // Mode
    pub(crate) const ONSET: Range<usize> = top::MODE.start..top::MODE.start + 1;
    pub(crate) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
    // Retention time
    pub(crate) const ABSOLUTE: Range<usize> =
        top::RETENTION_TIME.start..top::RETENTION_TIME.start + 1;
    pub(crate) const RELATIVE: Range<usize> = ABSOLUTE.end..ABSOLUTE.end + 1;
    pub(crate) const ADJUSTED: Range<usize> = RELATIVE.end..RELATIVE.end + 1;
    // Chain length
    pub(crate) const ECL: Range<usize> = top::CHAIN_LENGTH.start..top::CHAIN_LENGTH.start + 1;
    pub(crate) const FCL: Range<usize> = ECL.end..ECL.end + 1;
    pub(crate) const ECN: Range<usize> = FCL.end..FCL.end + 1;
}
