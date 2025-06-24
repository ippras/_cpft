use crate::{
    app::{
        panes::{MARGIN, distance::ID_SOURCE},
        states::distance::Settings,
    },
    r#const::*,
    utils::polars::SeriesExt,
};
use egui::{Frame, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_ext::ResponseExt;
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
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
    pub(super) fn show(&mut self, ui: &mut Ui) {
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
                ui.heading(ui.localize("Mode"))
                    .on_hover_localized("Mode.hover");
            }
            (0, top::FATTY_ACID) => {
                ui.heading(ui.localize("FattyAcid"))
                    .on_hover_localized("FattyAcid.abbreviation");
            }
            (0, top::DISTANCE) => {
                ui.heading(ui.localize("Distance"));
            }
            // Bottom
            (1, bottom::ONSET) => {
                ui.heading(ui.localize("OnsetTemperature.abbreviation"))
                    .on_hover_localized("OnsetTemperature");
            }
            (1, bottom::STEP) => {
                ui.heading(ui.localize("TemperatureStep.abbreviation"))
                    .on_hover_localized("TemperatureStep")
                    .on_hover_localized("TemperatureStep.hover");
            }
            (1, bottom::FROM) => {
                ui.heading(ui.localize("From"))
                    .on_hover_localized("From.hover");
            }
            (1, bottom::TO) => {
                ui.heading(ui.localize("To")).on_hover_localized("To.hover");
            }
            (1, bottom::RETENTION_TIME) => {
                ui.heading(ui.localize("RetentionTimeDistance.abbreviation"))
                    .on_hover_localized("RetentionTimeDistance")
                    .on_hover_localized("RetentionTimeDistance.hover");
            }
            (1, bottom::EQUIVALENT_CHAIN_LENGTH) => {
                ui.heading(ui.localize("EquivalentChainLengthDistance.abbreviation"))
                    .on_hover_localized("EquivalentChainLengthDistance")
                    .on_hover_localized("EquivalentChainLengthDistance.hover");
            }
            (1, bottom::ALPHA) => {
                ui.heading(ui.localize("Alpha.abbreviation"))
                    .on_hover_localized("Alpha")
                    .on_hover_localized("Alpha.hover");
            }
            (1, bottom::EUCLIDEAN) => {
                ui.heading(ui.localize("EuclideanDistance.abbreviation"))
                    .on_hover_localized("EuclideanDistance")
                    .on_hover_localized("EuclideanDistance.hover");
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
                        ui.heading(ui.localize("DeadTime"));
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
                        .ok_or(polars_err!(NoData: "FattyAcid.From[{row}]"))?,
                );
            }
            (row, bottom::TO) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .struct_()?
                        .field_by_name(TO)?
                        .str()?
                        .get(row)
                        .ok_or(polars_err!(NoData: "FattyAcid.To[{row}]"))?,
                );
            }
            (row, bottom::RETENTION_TIME) => {
                let retention_time = self.data_frame[RETENTION_TIME].struct_()?;
                ui.label(retention_time.field_by_name(DELTA)?.str_f64(row)?)
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(format!(
                            "{} - {}",
                            retention_time.field_by_name(TO)?.str_f64(row)?,
                            retention_time.field_by_name(FROM)?.str_f64(row)?
                        ));
                        Ok(())
                    })?;
            }
            (row, bottom::EQUIVALENT_CHAIN_LENGTH) => {
                let ecl = self.data_frame[EQUIVALENT_CHAIN_LENGTH].struct_()?;
                ui.label(ecl.field_by_name(DELTA)?.str_f64(row)?)
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(format!(
                            "{} - {}",
                            ecl.field_by_name(TO)?.str_f64(row)?,
                            ecl.field_by_name(FROM)?.str_f64(row)?
                        ));
                        Ok(())
                    })?;
            }
            (row, bottom::ALPHA) => {
                ui.label(
                    self.data_frame[ALPHA]
                        .as_materialized_series()
                        .str_f64(row)?,
                )
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    let retention_time = self.data_frame[RETENTION_TIME].struct_()?;
                    let dead_time = self.data_frame[DEAD_TIME].get(row)?.str_value();
                    ui.label(format!(
                        "({} - {dead_time}) / ({} - {dead_time})",
                        retention_time.field_by_name(TO)?.str_f64(row)?,
                        retention_time.field_by_name(FROM)?.str_f64(row)?
                    ));
                    Ok(())
                })?;
            }
            (row, bottom::EUCLIDEAN) => {
                ui.label(
                    self.data_frame[EUCLIDEAN]
                        .as_materialized_series()
                        .str_f64(row)?,
                )
                .try_on_hover_ui(|ui| -> PolarsResult<()> {
                    let retention_time = self.data_frame[RETENTION_TIME].struct_()?;
                    let ecl = self.data_frame[EQUIVALENT_CHAIN_LENGTH].struct_()?;
                    ui.label(format!(
                        "√({} - {})^2 + ({} - {})^2",
                        retention_time.field_by_name(TO)?.str_f64(row)?,
                        retention_time.field_by_name(FROM)?.str_f64(row)?,
                        ecl.field_by_name(TO)?.str_f64(row)?,
                        ecl.field_by_name(FROM)?.str_f64(row)?
                    ));
                    Ok(())
                })?;
            }
            _ => {}
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
                _ = self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(super) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 2;
    pub(super) const DISTANCE: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 4;
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
    pub(super) const ALPHA: Range<usize> =
        EQUIVALENT_CHAIN_LENGTH.end..EQUIVALENT_CHAIN_LENGTH.end + 1;
    pub(super) const EUCLIDEAN: Range<usize> = ALPHA.end..ALPHA.end + 1;
}
