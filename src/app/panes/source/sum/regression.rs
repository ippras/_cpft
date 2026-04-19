use crate::{
    app::{
        panes::MARGIN,
        states::source::{ID_SOURCE, Settings},
        widgets::array::{BooleanArray, Float64Array},
    },
    r#const::{
        ANY, DEAD_TIME, EM_DASH, MODE, ONSET_TEMPERATURE, REGRESSION, RETENTION_TIME,
        TEMPERATURE_STEP,
    },
    utils::egui::ToWidgetText,
};
use const_format::formatcp;
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::{FATTY_ACID, INDEX};
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const NUM_COLUMNS: usize = top::REGRESSION.end;

const TOP: &[Range<usize>] = &[
    top::INDEX,
    top::MODE,
    top::FATTY_ACID,
    top::RETENTION_TIME,
    top::DEAD_TIME,
    top::REGRESSION,
];

/// Regression widget
pub struct Regression<'a> {
    data_frame: &'a DataFrame,
    settings: &'a mut Settings,
}

impl<'a> Regression<'a> {
    pub fn new(data_frame: &'a DataFrame, settings: &'a mut Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let id_salt = Id::new(ID_SOURCE).with(REGRESSION);
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
            .show(ui, self)
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
                ui.heading(ui.localize(MODE)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MODE}.hover"));
                });
            }
            (0, top::FATTY_ACID) => {
                ui.heading(ui.localize(FATTY_ACID)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{FATTY_ACID}.hover"));
                });
            }
            (0, top::RETENTION_TIME) => {
                ui.heading(ui.localize(RETENTION_TIME)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{RETENTION_TIME}.hover"));
                });
            }
            (0, top::DEAD_TIME) => {
                ui.heading(ui.localize(DEAD_TIME)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{DEAD_TIME}.hover"));
                });
            }
            (0, top::REGRESSION) => {
                ui.heading(ui.localize(REGRESSION)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{REGRESSION}.hover"));
                });
            }
            // Bottom
            (1, bottom::mode::ONSET) => {
                ui.heading(ui.localize(formatcp!("{ONSET_TEMPERATURE}.abbreviation")))
                    .on_hover_ui(|ui| {
                        ui.localize(formatcp!("{ONSET_TEMPERATURE}.hover"));
                    });
            }
            (1, bottom::mode::STEP) => {
                ui.heading(ui.localize(formatcp!("{TEMPERATURE_STEP}.abbreviation")))
                    .on_hover_ui(|ui| {
                        ui.localize(formatcp!("{TEMPERATURE_STEP}.hover"));
                    });
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
        if let Some(true) = self.data_frame[REGRESSION]
            .struct_()?
            .field_by_name(ANY)?
            .bool()?
            .get(row)
        {
            ui.visuals_mut().override_text_color = Some(ui.visuals().strong_text_color());
        }
        match (row, column) {
            (row, top::INDEX) => {
                ui.label(row.to_string());
            }
            (row, bottom::mode::ONSET) => {
                ui.label(
                    self.data_frame[MODE]
                        .struct_()?
                        .field_by_name(ONSET_TEMPERATURE)?
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::mode::STEP) => {
                ui.label(
                    self.data_frame[MODE]
                        .struct_()?
                        .field_by_name(TEMPERATURE_STEP)?
                        .get(row)?
                        .str_value(),
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
            (row, top::RETENTION_TIME) => {
                Float64Array::builder()
                    .series(self.data_frame[RETENTION_TIME].as_materialized_series())
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, top::DEAD_TIME) => {
                let text = self.data_frame[DEAD_TIME].f64()?.get(row).to_widget_text();
                ui.label(text);
            }
            (row, top::REGRESSION) => {
                BooleanArray::builder()
                    .series(self.data_frame[REGRESSION].as_materialized_series())
                    .row(row)
                    .build()
                    .show(ui)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl TableDelegate for Regression<'_> {
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

impl Widget for Regression<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 2;
    pub(super) const FATTY_ACID: Range<usize> = MODE.end..MODE.end + 1;
    pub(super) const RETENTION_TIME: Range<usize> = FATTY_ACID.end..FATTY_ACID.end + 1;
    pub(super) const DEAD_TIME: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
    pub(super) const REGRESSION: Range<usize> = DEAD_TIME.end..DEAD_TIME.end + 1;
}

mod bottom {
    use super::*;

    // MODE
    pub(super) mod mode {
        use super::*;

        const TOP: Range<usize> = top::MODE;

        pub(in super::super) const ONSET: Range<usize> = TOP.start..TOP.start + 1;
        pub(in super::super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;
    }
}
