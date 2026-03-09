use crate::{
    app::{
        panes::MARGIN,
        states::source::{ID_SOURCE, State},
    },
    r#const::*,
    utils::polars::SeriesExt as _,
};
use egui::{Color32, Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_ext::ResponseExt;
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use lipid::prelude::*;
use polars::prelude::*;
use polars_utils::format_list;
use std::{borrow::Cow, ops::Range};
use tracing::instrument;

pub(crate) const NUM_COLUMNS: usize = top::DERIVATIVE.end;

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
            (0, top::RETENTION_TIME) => {
                ui.heading(ui.localize("RetentionTime"))
                    .on_hover_localized("RetentionTime.abbreviation")
                    .on_hover_localized("RetentionTime.hover");
            }
            (0, top::TEMPERATURE) => {
                ui.heading(ui.localize("Temperature"))
                    .on_hover_localized("Temperature.abbreviation")
                    .on_hover_localized("Temperature.hover");
            }
            (0, top::CHAIN_LENGTH) => {
                ui.heading(ui.localize("ChainLength"))
                    .on_hover_localized("ChainLength.hover");
            }
            (0, top::MASS) => {
                ui.heading(ui.localize("Mass"))
                    .on_hover_localized("Mass.hover");
            }
            (0, top::DERIVATIVE) => {
                ui.heading(ui.localize("Derivative"))
                    .on_hover_localized("Derivative.hover");
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
            (1, bottom::ABSOLUTE) => {
                ui.heading(ui.localize("AbsoluteRetentionTime"))
                    .on_hover_localized("AbsoluteRetentionTime.hover");
            }
            (1, bottom::RELATIVE) => {
                ui.heading(ui.localize("RelativeRetentionTime"))
                    .on_hover_localized("RelativeRetentionTime.hover");
            }
            (1, bottom::DELTA) => {
                ui.heading(ui.localize("DeltaRetentionTime"))
                    .on_hover_localized("DeltaRetentionTime.hover");
            }
            (1, bottom::ECL) => {
                ui.heading(ui.localize("EquivalentChainLength.abbreviation"))
                    .on_hover_localized("EquivalentChainLength");
            }
            (1, bottom::FCL) => {
                ui.heading(ui.localize("FractionalChainLength.abbreviation"))
                    .on_hover_localized("FractionalChainLength");
            }
            (1, bottom::ECN) => {
                ui.heading(ui.localize("EquivalentCarbonNumber.abbreviation"))
                    .on_hover_localized("EquivalentCarbonNumber");
            }
            (1, bottom::SLOPE) => {
                ui.heading(ui.localize("Slope"))
                    .on_hover_localized("Slope.hover");
            }
            (1, bottom::ANGLE) => {
                ui.heading(ui.localize("Angle"))
                    .on_hover_localized("Slope.hover");
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
            (row, top::FATTY_ACID) => {
                ui.label(
                    self.data_frame[FATTY_ACID]
                        .str()?
                        .get(row)
                        .unwrap_or(EM_DASH),
                );
            }
            (row, bottom::ABSOLUTE) => {
                let absolute_series = self.data_frame[RETENTION_TIME]
                    .struct_()?
                    .field_by_name(ABSOLUTE)?;
                let mean_series = absolute_series.struct_()?.field_by_name(MEAN)?;
                let standard_deviation_series = absolute_series
                    .struct_()?
                    .field_by_name(STANDARD_DEVIATION)?;
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
                    .map_or(Cow::Borrowed(EM_DASH), |mean| mean.to_string().into());
                ui.label(text)
                    .try_on_hover_ui(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let Some(mean) = mean_series.f64()?.get(row) else {
                            polars_bail!(NoData: "Mean[{row}]");
                        };
                        let Some(standard_deviation) = standard_deviation_series.f64()?.get(row)
                        else {
                            polars_bail!(NoData: "StandardDeviation[{row}]");
                        };
                        ui.heading(ui.localize("StandardDeviation"));
                        ui.label(format!("{mean} ±{standard_deviation}"));
                        Ok(())
                    })?
                    .try_on_hover_ui(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let Some(sample) = absolute_series
                            .struct_()?
                            .field_by_name("Sample")?
                            .list()?
                            .get_as_series(row)
                        else {
                            polars_bail!(NoData: "Sample[{row}]");
                        };
                        ui.heading(ui.localize("Sample"));
                        ui.label(format_list!(sample.iter()));
                        Ok(())
                    })?;
            }
            (row, bottom::RELATIVE) => {
                ui.label(
                    self.data_frame[RETENTION_TIME]
                        .struct_()?
                        .field_by_name(RELATIVE)?
                        .str_f64(row)?,
                );
            }
            (row, bottom::DELTA) => {
                ui.label(
                    self.data_frame[RETENTION_TIME]
                        .struct_()?
                        .field_by_name(DELTA)?
                        .str_f64(row)?,
                );
            }
            (row, top::TEMPERATURE) => {
                ui.label(
                    self.data_frame[TEMPERATURE]
                        .as_materialized_series()
                        .str_f64(row)?,
                );
            }
            (row, bottom::ECL) => {
                ui.label(
                    self.data_frame[CHAIN_LENGTH]
                        .struct_()?
                        .field_by_name(EQUIVALENT_CHAIN_LENGTH)?
                        .str_f64(row)?,
                );
            }
            (row, bottom::FCL) => {
                ui.label(
                    self.data_frame[CHAIN_LENGTH]
                        .struct_()?
                        .field_by_name(FRACTIONAL_CHAIN_LENGTH)?
                        .str_f64(row)?,
                );
            }
            (row, bottom::ECN) => {
                let ecn_series = self.data_frame[CHAIN_LENGTH]
                    .struct_()?
                    .field_by_name(EQUIVALENT_CARBON_NUMBER)?;
                let text = ecn_series.str_value(row)?;
                ui.label(text);
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
            (row, bottom::SLOPE) => {
                ui.label(
                    self.data_frame[DERIVATIVE]
                        .struct_()?
                        .field_by_name(SLOPE)?
                        .str_f64(row)?,
                );
            }
            (row, bottom::ANGLE) => {
                ui.label(
                    self.data_frame[DERIVATIVE]
                        .struct_()?
                        .field_by_name(ANGLE)?
                        .str_f64(row)?,
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
                _ = self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
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
