use crate::app::{
    panes::{MARGIN, distance::ID_SOURCE},
    states::distance::Settings,
};
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
use egui_l20n::UiExt as _;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const NUM_COLUMNS: usize = top::EUCLIDEAN_DISTANCE.end;

const TOP: &[Range<usize>] = &[
    top::MODE,
    top::ALPHA,
    top::EQUIVALENT_CHAIN_LENGTH,
    top::EUCLIDEAN_DISTANCE,
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
            .num_sticky_cols(self.settings.sticky)
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
                ui.heading("Index");
                // ui.heading(ui.localize("Mode")).on_hover_ui(|ui| {
                //     ui.localize("Mode.hover");
                // });
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
            (0, top::EUCLIDEAN_DISTANCE) => {
                ui.heading(ui.localize("EuclideanDistance"))
                    .on_hover_ui(|ui| {
                        ui.localize("EuclideanDistance.hover");
                    });
            }
            // Bottom
            (1, column) => {
                // self.data_frame.get_col
                ui.heading(ui.localize("OnsetTemperature.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.localize("OnsetTemperature.hover");
                    });
            }
            // (1, bottom::ONSET) => {
            //     ui.heading(ui.localize("onset-temperature.abbreviation"))
            //         .on_hover_localized("onset-temperature");
            // }
            // (1, bottom::STEP) => {
            //     ui.heading(ui.localize("temperature-step.abbreviation"))
            //         .on_hover_localized("temperature-step")
            //         .on_hover_localized("temperature-step.hover");
            // }
            // (1, bottom::FROM) => {
            //     ui.heading(ui.localize("from"));
            // }
            // (1, bottom::TO) => {
            //     ui.heading(ui.localize("to"));
            // }
            // (1, bottom::RETENTION_TIME) => {
            //     ui.heading(ui.localize("retention-time-distance.abbreviation"))
            //         .on_hover_localized("retention-time-distance")
            //         .on_hover_localized("retention-time-distance.hover");
            // }
            // (1, bottom::ECL) => {
            //     ui.heading(ui.localize("equivalent-chain-length-distance.abbreviation"))
            //         .on_hover_localized("equivalent-chain-length-distance")
            //         .on_hover_localized("equivalent-chain-length-distance.hover");
            // }
            // (1, bottom::EUCLIDEAN) => {
            //     ui.heading(ui.localize("euclidean-distance.abbreviation"))
            //         .on_hover_localized("euclidean-distance")
            //         .on_hover_localized("euclidean-distance.hover");
            // }
            // (1, bottom::ALPHA) => {
            //     ui.heading(ui.localize("alpha.abbreviation"))
            //         .on_hover_localized("alpha")
            //         .on_hover_localized("alpha.hover");
            // }
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
            (row, top::MODE) => {
                ui.label(row.to_string());
                // .try_on_hover_ui(|ui| -> PolarsResult<()> {
                //     ui.heading(ui.localize("DeadTime"));
                //     ui.label(self.data_frame[DEAD_TIME].get(row)?.str_value());
                //     Ok(())
                // })?;
            }
            _ => {}
        }
        Ok(())
    }

    // pub(crate) fn _show(self, ui: &mut Ui) -> Response {
    //     let mut response = ui.response();
    //     let height = ui.text_style_height(&TextStyle::Body);
    //     let width = ui.spacing().combo_width;
    //     ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
    //     response.rect = TableBuilder::new(ui)
    //         .resizable(true)
    //         .striped(true)
    //         .column(Column::auto().resizable(true))
    //         .columns(
    //             Column::remainder().at_least(width / 2.0),
    //             // .auto_size_this_frame(self.auto_size),
    //             self.data_frame.width(),
    //         )
    //         .header(height, |mut row| {
    //             for name in self.data_frame.get_column_names_str() {
    //                 row.col(|ui| {
    //                     ui.heading(name);
    //                 });
    //             }
    //         })
    //         .body(|body| {
    //             body.rows(height, self.data_frame.height(), |mut row| {
    //                 let index = row.index();
    //                 let mut iter = self.data_frame.iter();
    //                 if let Some(series) = iter.next() {
    //                     row.col(|ui| {
    //                         let text = series.get(index).unwrap().str_value();
    //                         ui.label(text);
    //                     });
    //                 }
    //                 for series in iter {
    //                     row.col(|ui| {
    //                         // let value = series.f64().unwrap().get(index).unwrap();
    //                         // let sign = Sign::from(value);
    //                         // let mut color = ui.style().visuals.text_color();
    //                         // if self.chaddock {
    //                         //     color = sign.chaddock().color(color);
    //                         // } else {
    //                         //     color = sign.color(color);
    //                         // }
    //                         let text = series.str_f64(index).unwrap();
    //                         ui.label(text);
    //                     });
    //                 }
    //             });
    //         })
    //         .inner_rect;
    //     response
    // }
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

    pub(super) const MODE: Range<usize> = 0..1;
    pub(super) const ALPHA: Range<usize> = MODE.end..MODE.end + 4;
    pub(super) const EQUIVALENT_CHAIN_LENGTH: Range<usize> = ALPHA.end..ALPHA.end + 4;
    pub(super) const EUCLIDEAN_DISTANCE: Range<usize> =
        EQUIVALENT_CHAIN_LENGTH.end..EQUIVALENT_CHAIN_LENGTH.end + 4;
}

// mod bottom {
//     use super::*;

//     pub(super) const ONSET: Range<usize> = top::ALPHA.start..top::ALPHA.start + 1;
//     pub(super) const STEP: Range<usize> = ONSET.end..ONSET.end + 1;

//     pub(super) const FROM: Range<usize> = top::FATTY_ACID.start..top::FATTY_ACID.start + 1;
//     pub(super) const TO: Range<usize> = FROM.end..FROM.end + 1;

//     pub(super) const RETENTION_TIME: Range<usize> = top::DISTANCE.start..top::DISTANCE.start + 1;
//     pub(super) const ECL: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
//     pub(super) const EUCLIDEAN: Range<usize> = ECL.end..ECL.end + 1;
//     pub(super) const ALPHA: Range<usize> = EUCLIDEAN.end..EUCLIDEAN.end + 1;
// }
