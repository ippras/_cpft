use crate::{
    app::{
        panes::{MARGIN, distance::ID_SOURCE},
        states::distance::Settings,
        widgets::array::Float64Array,
    },
    r#const::{
        ALPHA, EQUIVALENT_CHAIN_LENGTH, EUCLIDEAN, MAXIMUM, MEAN, MEDIAN, MINIMUM, MODE,
        ONSET_TEMPERATURE, TEMPERATURE_STEP,
    },
};
use const_format::formatcp;
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
use egui_l20n::prelude::*;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const NUM_COLUMNS: usize = top::EUCLIDEAN.end;

const TOP: &[Range<usize>] = &[
    top::MODE,
    top::ALPHA,
    top::EQUIVALENT_CHAIN_LENGTH,
    top::EUCLIDEAN,
];

/// Sum widget
pub(crate) struct Sum<'a> {
    data_frame: &'a DataFrame,
    settings: &'a mut Settings,
}

impl<'a> Sum<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a mut Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(super) fn show(&mut self, ui: &mut Ui) -> Response {
        let id_salt = Id::new(ID_SOURCE).with("Sum");
        if self.settings.reset_sum {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.settings.reset_sum = false;
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
            .num_sticky_cols(2)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self)
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, top::MODE) => {
                ui.heading(ui.localize(MODE)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MODE}.hover"));
                });
            }
            (0, top::ALPHA) => {
                ui.heading(ui.localize(ALPHA)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{ALPHA}.hover"));
                });
            }
            (0, top::EQUIVALENT_CHAIN_LENGTH) => {
                ui.heading(ui.localize(EQUIVALENT_CHAIN_LENGTH))
                    .on_hover_ui(|ui| {
                        ui.localize(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.hover"));
                    });
            }
            (0, top::EUCLIDEAN) => {
                ui.heading(ui.localize("EuclideanDistance"))
                    .on_hover_ui(|ui| {
                        ui.localize("EuclideanDistance.hover");
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
            (
                1,
                bottom::alpha::MAX | bottom::equivalent_chain_length::MAX | bottom::euclidean::MAX,
            ) => {
                ui.heading(ui.localize(MAXIMUM)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MAXIMUM}.hover"));
                });
            }
            (
                1,
                bottom::alpha::MEAN
                | bottom::equivalent_chain_length::MEAN
                | bottom::euclidean::MEAN,
            ) => {
                ui.heading(ui.localize(MEAN)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MEAN}.hover"));
                });
            }
            (
                1,
                bottom::alpha::MEDIAN
                | bottom::equivalent_chain_length::MEDIAN
                | bottom::euclidean::MEDIAN,
            ) => {
                ui.heading(ui.localize(MEDIAN)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MEDIAN}.hover"));
                });
            }
            (
                1,
                bottom::alpha::MIN | bottom::equivalent_chain_length::MIN | bottom::euclidean::MIN,
            ) => {
                ui.heading(ui.localize(MINIMUM)).on_hover_ui(|ui| {
                    ui.localize(formatcp!("{MINIMUM}.hover"));
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
            (row, bottom::alpha::MAX) => {
                Float64Array::builder()
                    .series(&self.data_frame[ALPHA].struct_()?.field_by_name(MAXIMUM)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::alpha::MEAN) => {
                Float64Array::builder()
                    .series(&self.data_frame[ALPHA].struct_()?.field_by_name(MEAN)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::alpha::MEDIAN) => {
                Float64Array::builder()
                    .series(&self.data_frame[ALPHA].struct_()?.field_by_name(MEDIAN)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::alpha::MIN) => {
                Float64Array::builder()
                    .series(&self.data_frame[ALPHA].struct_()?.field_by_name(MINIMUM)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::equivalent_chain_length::MAX) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EQUIVALENT_CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(MAXIMUM)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::equivalent_chain_length::MEAN) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EQUIVALENT_CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(MEAN)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::equivalent_chain_length::MEDIAN) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EQUIVALENT_CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(MEDIAN)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::equivalent_chain_length::MIN) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EQUIVALENT_CHAIN_LENGTH]
                            .struct_()?
                            .field_by_name(MINIMUM)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::euclidean::MAX) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EUCLIDEAN]
                            .struct_()?
                            .field_by_name(MAXIMUM)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::euclidean::MEAN) => {
                Float64Array::builder()
                    .series(&self.data_frame[EUCLIDEAN].struct_()?.field_by_name(MEAN)?)
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::euclidean::MEDIAN) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EUCLIDEAN]
                            .struct_()?
                            .field_by_name(MEDIAN)?,
                    )
                    .row(row)
                    .mean(self.settings.mean)
                    .standard_deviation(self.settings.standard_deviation)
                    .build()
                    .show(ui)?;
            }
            (row, bottom::euclidean::MIN) => {
                Float64Array::builder()
                    .series(
                        &self.data_frame[EUCLIDEAN]
                            .struct_()?
                            .field_by_name(MINIMUM)?,
                    )
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

impl TableDelegate for Sum<'_> {
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

impl Widget for Sum<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

mod top {
    use super::*;

    pub(super) const MODE: Range<usize> = 0..2;
    pub(super) const ALPHA: Range<usize> = MODE.end..MODE.end + 4;
    pub(super) const EQUIVALENT_CHAIN_LENGTH: Range<usize> = ALPHA.end..ALPHA.end + 4;
    pub(super) const EUCLIDEAN: Range<usize> =
        EQUIVALENT_CHAIN_LENGTH.end..EQUIVALENT_CHAIN_LENGTH.end + 4;
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

    // ALPHA
    pub(super) mod alpha {
        use super::*;

        const TOP: Range<usize> = top::ALPHA;

        pub(in super::super) const MAX: Range<usize> = TOP.start..TOP.start + 1;
        pub(in super::super) const MEAN: Range<usize> = MAX.end..MAX.end + 1;
        pub(in super::super) const MEDIAN: Range<usize> = MEAN.end..MEAN.end + 1;
        pub(in super::super) const MIN: Range<usize> = MEDIAN.end..MEDIAN.end + 1;
    }

    // EQUIVALENT_CHAIN_LENGTH
    pub(super) mod equivalent_chain_length {
        use super::*;

        const TOP: Range<usize> = top::EQUIVALENT_CHAIN_LENGTH;

        pub(in super::super) const MAX: Range<usize> = TOP.start..TOP.start + 1;
        pub(in super::super) const MEAN: Range<usize> = MAX.end..MAX.end + 1;
        pub(in super::super) const MEDIAN: Range<usize> = MEAN.end..MEAN.end + 1;
        pub(in super::super) const MIN: Range<usize> = MEDIAN.end..MEDIAN.end + 1;
    }

    // EUCLIDEAN_DISTANCE
    pub(super) mod euclidean {
        use super::*;

        const TOP: Range<usize> = top::EUCLIDEAN;

        pub(in super::super) const MAX: Range<usize> = TOP.start..TOP.start + 1;
        pub(in super::super) const MEAN: Range<usize> = MAX.end..MAX.end + 1;
        pub(in super::super) const MEDIAN: Range<usize> = MEAN.end..MEAN.end + 1;
        pub(in super::super) const MIN: Range<usize> = MEDIAN.end..MEDIAN.end + 1;
    }
}
