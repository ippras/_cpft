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
use itertools::Itertools;
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::option::DisplayOption;
use std::{fmt::Display, iter::zip, ops::Range};
use tracing::instrument;

pub(crate) const NUM_COLUMNS: usize = top::DISTANCE.end;

const TOP: &[Range<usize>] = &[
    top::INDEX,
    top::MODE,
    top::FATTY_ACID,
    top::DEAD_TIME,
    top::DISTANCE,
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
            (0, top::DEAD_TIME) => {
                ui.heading(ui.localize(DEAD_TIME))
                    .on_hover_localized(formatcp!("{DEAD_TIME}.abbreviation"))
                    .on_hover_localized(formatcp!("{DEAD_TIME}.hover"));
            }
            (0, top::DISTANCE) => {
                ui.heading(ui.localize(DISTANCE));
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
        // ui.visuals_mut().override_text_color = Some(ui.visuals().strong_text_color());
        if let Some(true) = self.data_frame[formatcp!("_{ERROR}")].bool()?.get(row) {
            ui.visuals_mut().override_text_color = Some(ui.visuals().error_fg_color);
        } else if let Some(true) = self.data_frame[formatcp!("_{WARNING}")].bool()?.get(row) {
            ui.visuals_mut().override_text_color = Some(ui.visuals().warn_fg_color);
        }
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
            (row, top::DEAD_TIME) => {
                let dead_time = self.dead_time(row)?;
                ui.label(dead_time.to_string());
            }
            (row, bottom::FROM) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .struct_()?
                        .field_by_name(FROM)?
                        .str()?
                        .get(row)
                        .ok_or(polars_err!(NoData: "{FATTY_ACID}.{FROM}[{row}]"))?,
                )
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(RETENTION_TIME));
                    let text = format_array(&self.float64(row, RETENTION_TIME, FROM)?);
                    ui.label(text);
                    Ok(())
                })?
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(EQUIVALENT_CHAIN_LENGTH));
                    let text = format_array(&self.float64(row, EQUIVALENT_CHAIN_LENGTH, FROM)?);
                    ui.label(text);
                    Ok(())
                })?;
            }
            (row, bottom::TO) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .struct_()?
                        .field_by_name(TO)?
                        .str()?
                        .get(row)
                        .ok_or(polars_err!(NoData: "{FATTY_ACID}.{TO}[{row}]"))?,
                )
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(RETENTION_TIME));
                    let text = format_array(&self.float64(row, RETENTION_TIME, TO)?);
                    ui.label(text);
                    Ok(())
                })?
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(EQUIVALENT_CHAIN_LENGTH));
                    let text = format_array(&self.float64(row, EQUIVALENT_CHAIN_LENGTH, TO)?);
                    ui.label(text);
                    Ok(())
                })?;
            }
            (row, bottom::RETENTION_TIME) => {
                let retention_time = &self.data_frame[RETENTION_TIME];
                Float64Array::builder()
                    .series(&retention_time.struct_()?.field_by_name(DISTANCE)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        for (from, to) in zip(
                            &self.float64(row, RETENTION_TIME, FROM)?,
                            &self.float64(row, RETENTION_TIME, TO)?,
                        ) {
                            let to = to.display();
                            let from = from.display();
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
                            .field_by_name(DISTANCE)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        for (from, to) in zip(
                            &self.float64(row, EQUIVALENT_CHAIN_LENGTH, FROM)?,
                            &self.float64(row, EQUIVALENT_CHAIN_LENGTH, TO)?,
                        ) {
                            let to = to.display();
                            let from = from.display();
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
                        let dead_time = self.data_frame[DEAD_TIME].get(row)?.str_value();
                        for (from, to) in zip(
                            &self.float64(row, RETENTION_TIME, FROM)?,
                            &self.float64(row, RETENTION_TIME, TO)?,
                        ) {
                            let to = to.display();
                            let from = from.display();
                            ui.label(format!("({to} - {dead_time}) / ({from} - {dead_time})"));
                        }
                        Ok(())
                    })?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn dead_time(&self, row: usize) -> PolarsResult<f64> {
        let Some(dead_time) = self.data_frame[DEAD_TIME].f64()?.get(row) else {
            return Err(polars_err!(NoData: "{DEAD_TIME}[{row}]"));
        };
        Ok(dead_time)
    }

    fn float64(&self, row: usize, column: &str, field: &str) -> PolarsResult<Float64Chunked> {
        let Some(series) = self.data_frame[column]
            .struct_()?
            .field_by_name(field)?
            .struct_()?
            .field_by_name(ARRAY)?
            .array()?
            .get_as_series(row)
        else {
            return Err(polars_err!(NoData: "{column}.{field}.{ARRAY}[{row}]"));
        };
        Ok(series.f64()?.clone())
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

fn format_array(array: impl IntoIterator<Item = Option<impl Display>>) -> String {
    format!(
        "[{}]",
        array
            .into_iter()
            .format_with(", ", |item, f| f(&item.display()))
    )
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(super) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 2;
    pub(super) const DEAD_TIME: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 1;
    pub(super) const DISTANCE: Range<usize> = DEAD_TIME.end..DEAD_TIME.end + 3;
}

mod bottom {
    use super::*;

    // MODE
    pub(super) const ONSET: Range<usize> = top::MODE.start..top::MODE.start + 1;
    pub(super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
    // FATTY_ACID
    pub(super) const FROM: Range<usize> = top::FATTY_ACID.start..top::FATTY_ACID.start + 1;
    pub(super) const TO: Range<usize> = FROM.end..FROM.end + 1;
    // DISTANCE
    pub(super) const RETENTION_TIME: Range<usize> = top::DISTANCE.start..top::DISTANCE.start + 1;
    pub(super) const EQUIVALENT_CHAIN_LENGTH: Range<usize> =
        RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(super) const SELECTIVITY_FACTOR: Range<usize> =
        EQUIVALENT_CHAIN_LENGTH.end..EQUIVALENT_CHAIN_LENGTH.end + 1;
}
