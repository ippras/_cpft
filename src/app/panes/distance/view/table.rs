use crate::{
    app::{
        panes::{MARGIN, distance::ID_SOURCE},
        states::distance::Settings,
        widgets::array::Float64Array,
    },
    r#const::*,
};
use const_format::formatcp;
use egui::{Frame, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_ext::ResponseExt;
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::prelude::*;
use polars::prelude::*;
use std::{iter::zip, ops::Range};
use tracing::instrument;

pub(crate) const NUM_COLUMNS: usize = top::DISTANCE.end;

const TOP: &[Range<usize>] = &[top::INDEX, top::MODE, top::FATTY_ACID, top::DISTANCE];

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
        if self.settings.reset_table {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.settings.reset_table = false;
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
                ui.heading(HASH).on_hover_localized("Index");
            }
            (0, top::MODE) => {
                ui.heading(ui.localize(MODE))
                    .on_hover_localized(formatcp!("{MODE}.hover"));
            }
            (0, top::FATTY_ACID) => {
                ui.heading(ui.localize(FATTY_ACID))
                    .on_hover_localized(formatcp!("{FATTY_ACID}.abbreviation"));
            }
            (0, top::DISTANCE) => {
                ui.heading(ui.localize("Distance"));
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
            (1, bottom::FROM) => {
                ui.heading(ui.localize("From"))
                    .on_hover_localized("From.hover");
            }
            (1, bottom::TO) => {
                ui.heading(ui.localize("To")).on_hover_localized("To.hover");
            }
            (1, bottom::RETENTION_TIME) => {
                ui.heading(ui.localize(formatcp!("{RETENTION_TIME}.abbreviation")))
                    .on_hover_localized(RETENTION_TIME);
            }
            (1, bottom::EQUIVALENT_CHAIN_LENGTH) => {
                ui.heading(ui.localize(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.abbreviation")))
                    .on_hover_localized(EQUIVALENT_CHAIN_LENGTH);
            }
            (1, bottom::SELECTIVITY_FACTOR) => {
                ui.heading(ui.localize(formatcp!("{SELECTIVITY_FACTOR}.abbreviation")))
                    .on_hover_localized(SELECTIVITY_FACTOR);
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
                ui.label(row.to_string())
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading(ui.localize(DEAD_TIME));
                        ui.label(self.data_frame[DEAD_TIME].get(row)?.str_value());
                        Ok(())
                    })?;
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
            (row, bottom::FROM) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .struct_()?
                        .field_by_name(FROM)?
                        .str()?
                        .get(row)
                        .ok_or(polars_err!(NoData: "{FATTY_ACID}.{FROM}[{row}]"))?,
                );
            }
            (row, bottom::TO) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .struct_()?
                        .field_by_name(TO)?
                        .str()?
                        .get(row)
                        .ok_or(polars_err!(NoData: "{FATTY_ACID}.{TO}[{row}]"))?,
                );
            }
            (row, bottom::RETENTION_TIME) => {
                let retention_time = &self.data_frame[RETENTION_TIME];
                Float64Array::builder()
                    .series(&retention_time.struct_()?.field_by_name(DELTA)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let retention_times = self.retention_times(row)?;
                        for FromTo { from, to } in retention_times.into_no_null_iter()? {
                            ui.label(format!("{to} - {from}"));
                        }
                        Ok(())
                    })?;
            }
            (row, bottom::EQUIVALENT_CHAIN_LENGTH) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EQUIVALENT_CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(DELTA)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let equivalent_chain_lengths = self.equivalent_chain_lengths(row)?;
                        for FromTo { from, to } in equivalent_chain_lengths.into_no_null_iter()? {
                            ui.label(format!("{to} - {from}",));
                        }
                        Ok(())
                    })?;
            }
            (row, bottom::SELECTIVITY_FACTOR) => {
                Float64Array::builder()
                    .series(self.data_frame[SELECTIVITY_FACTOR].as_materialized_series())
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let retention_times = self.retention_times(row)?;
                        let dead_time = self.data_frame[DEAD_TIME].get(row)?.str_value();
                        for FromTo { from, to } in retention_times.into_no_null_iter()? {
                            ui.label(format!("({to} - {dead_time}) / ({from} - {dead_time})"));
                        }
                        Ok(())
                    })?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn equivalent_chain_lengths(&self, row: usize) -> PolarsResult<FromTo<Float64Chunked>> {
        self.array(row, EQUIVALENT_CHAIN_LENGTH)
    }

    fn retention_times(&self, row: usize) -> PolarsResult<FromTo<Float64Chunked>> {
        let r#struct = self.data_frame[RETENTION_TIME].struct_()?;
        let Some(to) = r#struct
            .field_by_name(TO)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{RETENTION_TIME}.{TO}.{ARRAY}[{row}]"));
        };
        let Some(from) = r#struct
            .field_by_name(FROM)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{RETENTION_TIME}.{FROM}.{ARRAY}[{row}]"));
        };
        Ok(FromTo {
            from: from.f64()?.fill_null_with_values(f64::NAN)?,
            to: to.f64()?.fill_null_with_values(f64::NAN)?,
        })
    }

    fn array(&self, row: usize, name: &str) -> PolarsResult<FromTo<Float64Chunked>> {
        let r#struct = self.data_frame[name].struct_()?;
        let Some(to) = r#struct
            .field_by_name(TO)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{name}.{TO}.{ARRAY}[{row}]"));
        };
        let Some(from) = r#struct
            .field_by_name(FROM)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{name}.{FROM}.{ARRAY}[{row}]"));
        };
        Ok(FromTo {
            from: from.f64()?.fill_null_with_values(f64::NAN)?,
            to: to.f64()?.fill_null_with_values(f64::NAN)?,
        })
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

struct FromTo<T> {
    from: T,
    to: T,
}

impl FromTo<Float64Chunked> {
    fn into_no_null_iter(&self) -> PolarsResult<impl Iterator<Item = FromTo<f64>>> {
        Ok(
            zip(self.from.into_no_null_iter(), self.to.into_no_null_iter())
                .map(|(from, to)| FromTo { from, to }),
        )
    }
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(super) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 2;
    pub(super) const DISTANCE: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 3;
}

mod bottom {
    use super::*;

    // MODE
    pub(super) const ONSET: Range<usize> = top::MODE.start..top::MODE.start + 1;
    pub(super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
    // FATTY_ACID
    pub(super) const FROM: Range<usize> = STEP.end..STEP.end + 1;
    pub(super) const TO: Range<usize> = FROM.end..FROM.end + 1;
    // DISTANCE
    pub(super) const RETENTION_TIME: Range<usize> = TO.end..TO.end + 1;
    pub(super) const EQUIVALENT_CHAIN_LENGTH: Range<usize> =
        RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(super) const SELECTIVITY_FACTOR: Range<usize> =
        EQUIVALENT_CHAIN_LENGTH.end..EQUIVALENT_CHAIN_LENGTH.end + 1;
}
