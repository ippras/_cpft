use self::{plot::PlotView, table::TableView};
use crate::{
    app::{
        computers::source::{
            Computed as SourceComputed, Key as SourceKey,
            format::{Computed as DisplayComputed, Key as DisplayKey},
            plot::{Computed as PlotComputed, Key as PlotKey},
            sum::{
                correlation::{Computed as CorrelationComputed, Key as CorrelationKey},
                regression::{Computed as RegressionComputed, Key as RegressionKey},
            },
        },
        panes::{
            Behavior, MARGIN,
            source::sum::{correlation::Correlation, regression::Regression},
        },
        states::source::{ID_SOURCE, State, View},
        widgets::buttons::{MetadataButton, ResetButton, ResizeButton, SettingsButton, ViewButton},
    },
    r#const::{
        ABSOLUTE, CORRELATION, DEAD_TIME, MEAN, MODE, ONSET_TEMPERATURE, REGRESSION,
        RETENTION_TIME, TEMPERATURE_STEP,
    },
    export,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use const_format::formatcp;
use egui::{
    Button, CentralPanel, CursorIcon, Frame, Id, MenuBar, Panel, Response, RichText, ScrollArea,
    TextStyle, Ui, Widget as _, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{FLOPPY_DISK, RULER, SIGMA, SLIDERS_HORIZONTAL, TABLE, TAG, X};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::{egui::MetadataWidget, polars::MetaDataFrame};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, from_fn};
use tracing::instrument;

/// Source pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frame: HashedMetaDataFrame,
    calculated: HashedDataFrame,
}

impl Pane {
    pub(crate) fn new(frame: HashedMetaDataFrame) -> Self {
        Self {
            id: None,
            frame,
            calculated: HashedDataFrame::EMPTY,
        }
    }

    pub(crate) fn title(&self) -> String {
        self.frame.meta.format(" ").to_string()
    }

    fn id(&self) -> impl Display {
        from_fn(|f| {
            if let Some(id) = self.id {
                write!(f, "{id:?}-")?;
            }
            write!(f, "{}", hash(&self.frame))
        })
    }
}

impl Pane {
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let id = *self.id.get_or_insert_with(|| ui.next_auto_id());
        let mut state = State::load(ui.ctx(), id);
        _ = self.init(ui, &mut state);
        let response = Panel::top(ui.auto_id_with("Pane"))
            .show_inside(ui, |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                if ui.button(RichText::new(X).heading()).clicked() {
                                    behavior.close = Some(tile_id);
                                }
                                ui.separator();
                                self.top(ui, &mut state)
                            })
                            .inner
                    })
                    .inner
            })
            .inner;
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()))
            .show_inside(ui, |ui| {
                self.central(ui, &mut state);
                self.windows(ui, &mut state);
            });
        if behavior.close == Some(tile_id) {
            state.remove(ui.ctx(), id);
        } else {
            state.store(ui.ctx(), id);
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    #[instrument(skip_all, err)]
    fn init(&mut self, ui: &mut Ui, state: &mut State) -> Result<()> {
        self.calculated = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SourceComputed>()
                .get(SourceKey::new(&self.frame.data, &state.settings))
                .clone()
        });
        state.settings.cache.onset_temperatures = self.calculated.data_frame[MODE]
            .struct_()?
            .field_by_name(ONSET_TEMPERATURE)?
            .f64()?
            .unique()?
            .sort(false)
            .into_no_null_iter()
            .collect::<Vec<_>>();
        state.settings.cache.temperature_steps = self.calculated.data_frame[MODE]
            .struct_()?
            .field_by_name(TEMPERATURE_STEP)?
            .f64()?
            .unique()?
            .sort(false)
            .into_no_null_iter()
            .collect::<Vec<_>>();
        state.settings.cache.fatty_acids = self.calculated.data_frame[FATTY_ACID]
            .unique_stable()?
            .fatty_acid()
            .fields()?
            .into_iter()
            .filter_map(|fatty_acid| fatty_acid.transpose())
            .collect::<PolarsResult<Vec<_>>>()?;
        Ok(())
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(TABLE).on_hover_text(ui.localize("Source"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.calculated.hash))
            .on_hover_ui(|ui| MetadataWidget::new(&self.frame.meta).show(ui))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        ResetButton::new(&mut state.settings.reset).ui(ui);
        ui.separator();
        ResizeButton::new(&mut state.settings.resizable).ui(ui);
        ui.separator();
        SettingsButton::new(&mut state.windows.open_settings).ui(ui);
        ui.separator();
        ViewButton::new(&mut state.settings.view).ui(ui);
        ui.separator();
        MetadataButton::new(&mut state.windows.open_metadata).ui(ui);
        ui.separator();
        self.sum_button(ui, state);
        ui.separator();
        self.save_button(ui, state);
        ui.separator();
        self.distance_button(ui, state);
        ui.separator();
        response
    }

    /// Sum button
    fn sum_button(&self, ui: &mut Ui, state: &mut State) {
        ui.menu_button(RichText::new(SIGMA).heading(), |ui| {
            ui.toggle_value(
                &mut state.windows.open_correlation,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Correlation")).heading(),
                ),
            )
            .on_hover_localized("Correlation.hover");
            ui.toggle_value(
                &mut state.windows.open_regression,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Regression")).heading(),
                ),
            )
            .on_hover_localized("Regression.hover");
        });
    }

    /// Save button
    fn save_button(&self, ui: &mut Ui, state: &State) {
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let name = self.frame.meta.format(".");
            if ui
                .button((FLOPPY_DISK, "RON"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.cpft.ron"));
                })
                .clicked()
            {
                let _ = self.save_ron(ui, state, &name);
            }
            if ui
                .button((FLOPPY_DISK, "CSV"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.cpft.csv"));
                })
                .clicked()
            {
                let _ = self.save_csv(ui, state, &name);
            }
            if ui
                .button((FLOPPY_DISK, "XLSX"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.cpft.xlsx"));
                })
                .clicked()
            {
                let _ = self.save_xlsx(ui, state, &name);
            }
        });
    }

    #[instrument(skip(self, ui, state), err)]
    fn save_ron(&self, ui: &mut Ui, state: &State, name: impl Debug + Display) -> Result<()> {
        // let data_frame = ui.memory_mut(|memory| {
        //     memory
        //         .caches
        //         .cache::<DisplayComputed>()
        //         .get(DisplayKey::new(&self.calculated, &state.settings))
        // });
        // let data = self
        //     .frame
        //     .data
        //     .data_frame
        //     .clone()
        //     .lazy()
        //     .select([
        //         col(MODE),
        //         col(FATTY_ACID),
        //         col(RETENTION_TIME)
        //             .struct_()
        //             .field_by_name(ABSOLUTE)
        //             .struct_()
        //             .field_by_name(SAMPLE)
        //             .name()
        //             .keep(),
        //         col(DEAD_TIME),
        //     ])
        //     .with_row_index("Index", None)
        //     .collect()?;
        let data = self
            .frame
            .data
            .data_frame
            .clone()
            .lazy()
            .select([
                col(MODE),
                col(FATTY_ACID),
                col(RETENTION_TIME),
                col(DEAD_TIME),
            ])
            .with_row_index("Index", None)
            .collect()?;
        let frame = MetaDataFrame::new(&self.frame.meta, data);
        // export::ron::save(&frame, &format!("{name}.cpft.ron"))?;
        Ok(())
    }

    #[instrument(skip(self, ui, state), err)]
    fn save_csv(&self, ui: &mut Ui, state: &State, name: impl Debug + Display) -> Result<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DisplayComputed>()
                .get(DisplayKey::new(&self.calculated, &state.settings))
                .clone()
        });
        let mut data = data_frame
            .lazy()
            .select([
                col(MODE).struct_().field_by_name("*"),
                col(FATTY_ACID),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(MEAN)
                    .name()
                    .keep(),
                col(DEAD_TIME),
            ])
            .with_row_index("Index", None)
            .collect()?;
        export::csv::save(&mut data, &format!("{name}.cpft.csv"))?;
        Ok(())
    }

    #[instrument(skip(self, ui, state), err)]
    fn save_xlsx(&self, ui: &mut Ui, state: &State, name: impl Debug + Display) -> Result<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DisplayComputed>()
                .get(DisplayKey::new(&self.calculated, &state.settings))
                .clone()
        });
        let mut data = data_frame
            .lazy()
            .select([
                col(MODE).struct_().field_by_name("*"),
                col(FATTY_ACID),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(MEAN)
                    .name()
                    .keep(),
                col(DEAD_TIME),
            ])
            .with_row_index("Index", None)
            .collect()?;
        // export::xlsx::save(&mut data, &format!("{name}.cpft.xlsx"))?;
        Ok(())
    }

    /// Distance button
    fn distance_button(&mut self, ui: &mut Ui, state: &mut State) {
        if ui
            .add_enabled(
                state.settings.view == View::Table,
                Button::new(RichText::new(RULER).heading()),
            )
            .clicked()
        {
            let data = self.calculated.clone();
            let meta = self.frame.meta.clone();
            let frame = HashedMetaDataFrame::new(meta, data);
            ui.data_mut(|data| data.insert_temp(Id::new("Distance"), frame));
        }
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        match state.settings.view {
            View::Plot => {
                let points = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<PlotComputed>()
                        .get(PlotKey::new(&self.calculated, &state.settings))
                        .clone()
                });
                PlotView::new(points, &state.settings).show(ui)
            }
            View::Table => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayComputed>()
                        .get(DisplayKey::new(&self.calculated, &state.settings))
                        .clone()
                });
                TableView::new(&data_frame, &mut state.settings).show(ui)
            }
        };
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.metadata_window(ui, state);
        self.settings_window(ui, state);
        self.sum_windows(ui, state);
    }

    fn metadata_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(formatcp!("{TAG} Source metadata"))
            .id(ui.auto_id_with(ID_SOURCE).with("Metadata"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_metadata)
            .show(ui.ctx(), |ui| {
                MetadataWidget::new(&self.frame.meta).show(ui);
            });
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(formatcp!("{SLIDERS_HORIZONTAL} Source settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui);
            });
    }

    fn sum_windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.correlation_window(ui, state);
        self.regression_window(ui, state);
    }

    fn correlation_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(formatcp!("{SIGMA} {CORRELATION}"))
            .id(ui.auto_id_with(ID_SOURCE).with(CORRELATION))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_correlation)
            .resizable(state.settings.resizable)
            .show(ui.ctx(), |ui| {
                Panel::top(ui.auto_id_with("Top")).show_inside(ui, |ui| {
                    MenuBar::new()
                        .ui(ui, |ui| {
                            ScrollArea::horizontal()
                                .show(ui, |ui| {
                                    ui.set_height(
                                        ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                    );
                                    ui.visuals_mut().button_frame = false;
                                    ResetButton::new(&mut state.settings.reset).ui(ui);
                                    ui.separator();
                                    ResizeButton::new(&mut state.settings.resizable).ui(ui);
                                    ui.separator();
                                })
                                .inner
                        })
                        .inner;
                });
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CorrelationComputed>()
                        .get(CorrelationKey::new(&self.calculated, &state.settings))
                        .clone()
                });
                Correlation::new(&data_frame, &mut state.settings).show(ui);
            });
    }

    fn regression_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(formatcp!("{SIGMA} {REGRESSION}"))
            .id(ui.auto_id_with(ID_SOURCE).with(REGRESSION))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_regression)
            .resizable(state.settings.resizable)
            .show(ui.ctx(), |ui| {
                Panel::top(ui.auto_id_with("Top")).show_inside(ui, |ui| {
                    MenuBar::new()
                        .ui(ui, |ui| {
                            ScrollArea::horizontal()
                                .show(ui, |ui| {
                                    ui.set_height(
                                        ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                    );
                                    ui.visuals_mut().button_frame = false;
                                    ResetButton::new(&mut state.settings.reset).ui(ui);
                                    ui.separator();
                                    ResizeButton::new(&mut state.settings.resizable).ui(ui);
                                    ui.separator();
                                })
                                .inner
                        })
                        .inner;
                });
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<RegressionComputed>()
                        .get(RegressionKey::new(&self.calculated, &state.settings))
                        .clone()
                });
                Regression::new(&data_frame, &mut state.settings).show(ui);
            });
    }
}

pub(crate) mod plot;
pub(crate) mod sum;
pub(crate) mod table;
