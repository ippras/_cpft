use crate::{
    app::{
        panes::{MARGIN, distance::ID_SOURCE},
        states::distance::Settings,
    },
    r#const::{
        ALPHA, EQUIVALENT_CHAIN_LENGTH, EUCLIDEAN, MODE, ONSET_TEMPERATURE, TEMPERATURE_STEP,
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
                ui.heading(ui.localize("Mode")).on_hover_ui(|ui| {
                    ui.localize("Mode.hover");
                });
            }
            (0, top::ALPHA) => {
                ui.heading(ui.localize("Alpha")).on_hover_ui(|ui| {
                    ui.localize("Alpha.hover");
                });
            }
            (0, top::EQUIVALENT_CHAIN_LENGTH) => {
                ui.heading(ui.localize("EquivalentChainLength"))
                    .on_hover_ui(|ui| {
                        ui.localize("EquivalentChainLength.hover");
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
            (
                1,
                bottom::alpha::MAX | bottom::equivalent_chain_length::MAX | bottom::euclidean::MAX,
            ) => {
                ui.heading(ui.localize("Maximum")).on_hover_ui(|ui| {
                    ui.localize("Maximum.hover");
                });
            }
            (
                1,
                bottom::alpha::MEAN
                | bottom::equivalent_chain_length::MEAN
                | bottom::euclidean::MEAN,
            ) => {
                ui.heading(ui.localize("Mean")).on_hover_ui(|ui| {
                    ui.localize("Mean.hover");
                });
            }
            (
                1,
                bottom::alpha::MEDIAN
                | bottom::equivalent_chain_length::MEDIAN
                | bottom::euclidean::MEDIAN,
            ) => {
                ui.heading(ui.localize("Median")).on_hover_ui(|ui| {
                    ui.localize("Median.hover");
                });
            }
            (
                1,
                bottom::alpha::MIN | bottom::equivalent_chain_length::MIN | bottom::euclidean::MIN,
            ) => {
                ui.heading(ui.localize("Minimum")).on_hover_ui(|ui| {
                    ui.localize("Minimum.hover");
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
                ui.label(
                    self.data_frame[formatcp!("{ALPHA}.Max")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::alpha::MEAN) => {
                ui.label(
                    self.data_frame[formatcp!("{ALPHA}.Mean")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::alpha::MEDIAN) => {
                ui.label(
                    self.data_frame[formatcp!("{ALPHA}.Median")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::alpha::MIN) => {
                ui.label(
                    self.data_frame[formatcp!("{ALPHA}.Min")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::equivalent_chain_length::MAX) => {
                ui.label(
                    self.data_frame[formatcp!("{EQUIVALENT_CHAIN_LENGTH}.Max")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::equivalent_chain_length::MEAN) => {
                ui.label(
                    self.data_frame[formatcp!("{EQUIVALENT_CHAIN_LENGTH}.Mean")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::equivalent_chain_length::MEDIAN) => {
                ui.label(
                    self.data_frame[formatcp!("{EQUIVALENT_CHAIN_LENGTH}.Median")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::equivalent_chain_length::MIN) => {
                ui.label(
                    self.data_frame[formatcp!("{EQUIVALENT_CHAIN_LENGTH}.Min")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::euclidean::MAX) => {
                ui.label(
                    self.data_frame[formatcp!("{EUCLIDEAN}.Max")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::euclidean::MEAN) => {
                ui.label(
                    self.data_frame[formatcp!("{EUCLIDEAN}.Mean")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::euclidean::MEDIAN) => {
                ui.label(
                    self.data_frame[formatcp!("{EUCLIDEAN}.Median")]
                        .get(row)?
                        .str_value(),
                );
            }
            (row, bottom::euclidean::MIN) => {
                ui.label(
                    self.data_frame[formatcp!("{EUCLIDEAN}.Min")]
                        .get(row)?
                        .str_value(),
                );
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
