use crate::{
    app::{
        panes::MARGIN,
        states::source::{ID_SOURCE, Settings},
        widgets::array::Array,
    },
    r#const::{CORRELATION, MODE, ONSET_TEMPERATURE, TEMPERATURE_STEP},
};
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
use egui_l20n::prelude::*;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const NUM_COLUMNS: usize = top::CORRELATION.end;

const TOP: &[Range<usize>] = &[top::MODE, top::CORRELATION];

/// Correlation widget
pub struct Correlation<'a> {
    data_frame: &'a DataFrame,
    settings: &'a mut Settings,
}

impl<'a> Correlation<'a> {
    pub fn new(data_frame: &'a DataFrame, settings: &'a mut Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let id_salt = Id::new(ID_SOURCE).with("Correlation");
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
            (0, top::MODE) => {
                ui.heading(ui.localize("Mode")).on_hover_ui(|ui| {
                    ui.localize("Mode.hover");
                });
            }
            (0, top::CORRELATION) => {
                ui.heading(ui.localize("PearsonCorrelation"))
                    .on_hover_ui(|ui| {
                        ui.localize("PearsonCorrelation.hover");
                    });
            }
            // Bottom
            (1, bottom::mode::ONSET) => {
                ui.heading(ui.localize("OnsetTemperature.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.localize("OnsetTemperature.hover");
                    });
            }
            (1, bottom::mode::STEP) => {
                ui.heading(ui.localize("TemperatureStep.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.localize("TemperatureStep.hover");
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
        match (row, column) {
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
            (row, top::CORRELATION) => {
                Array::builder()
                    .series(self.data_frame[CORRELATION].as_materialized_series())
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl TableDelegate for Correlation<'_> {
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

impl Widget for Correlation<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

mod top {
    use super::*;

    pub(super) const MODE: Range<usize> = 0..2;
    pub(super) const CORRELATION: Range<usize> = MODE.end..MODE.end + 1;
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
