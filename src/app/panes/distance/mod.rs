use self::{
    sum::Sum,
    view::{plot::PlotView, table::TableView},
};
use crate::{
    app::{
        computers::distance::{
            process::{Computed as ProcessComputed, Key as ProcessKey},
            sum::{Computed as SumComputed, Key as SumKey},
            view::{
                plot::{Computed as PlotComputed, Key as PlotKey},
                table::{Computed as TableComputed, Key as TableKey},
            },
        },
        panes::{Behavior, MARGIN},
        states::distance::{Settings, State, View},
        widgets::buttons::{MetadataButton, ResetButton, ResizeButton, SettingsButton, ViewButton},
    },
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Panel, Response, RichText, ScrollArea, TextStyle,
    Ui, Widget as _, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{FLOPPY_DISK, RULER, SIGMA, SLIDERS_HORIZONTAL, TAG, X};
use egui_tiles::{TileId, UiResponse};
use metadata::egui::MetadataWidget;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, from_fn};
use tracing::instrument;

const ID_SOURCE: &str = "Distance";

/// Distance pane
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
        self.init(ui, &mut state);
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

    // #[instrument(skip_all, err)]
    fn init(&mut self, ui: &mut Ui, state: &mut State) {
        self.calculated = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<ProcessComputed>()
                .get(ProcessKey::new(&self.frame.data, &state.settings))
                .clone()
        });
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui.heading(RULER).on_hover_localized("Distance");
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.calculated.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        let mut selected = false;
        ResetButton::new(&mut selected).ui(ui);
        if selected {
            state.settings.reset_table = selected;
            state.settings.reset_sum = selected;
        }
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
        self.save_button(ui);
        ui.separator();
        response
    }

    /// Save button
    fn save_button(&self, ui: &mut Ui) {
        // let name = format!("{}.distance.ipc", self.frame.frame.meta.title());
        // if ui
        //     .button(RichText::new(FLOPPY_DISK).heading())
        //     .on_hover_text(&name)
        //     .clicked()
        // {
        //     if let Err(error) = save(
        //         &name,
        //         MetaDataFrame::new(&self.frame.frame.meta, &mut self.target),
        //     ) {
        //         error!(%error);
        //     }
        // }
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let meta = &self.frame.meta;
            let name = meta.format(".");
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
                // let _ = self.save_ron(&name, meta);
            }
        });
    }

    /// Sum button
    fn sum_button(&self, ui: &mut Ui, state: &mut State) {
        ui.menu_button(RichText::new(SIGMA).heading(), |ui| {
            ui.toggle_value(
                &mut state.windows.open_sum,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Sum")).heading(),
                ),
            )
            .on_hover_localized("Sum.hover");
        });
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
                // Format
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<TableComputed>()
                        .get(TableKey::new(&self.calculated, &state.settings))
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
        self.sum_window(ui, state);
    }

    fn metadata_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{TAG} Distance metadata"))
            .id(ui.auto_id_with(ID_SOURCE).with("Metadata"))
            .constrain_to(ui.clip_rect())
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_metadata)
            .show(ui.ctx(), |ui| {
                MetadataWidget::new(&self.frame.meta).show(ui);
            });
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Distance settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .constrain_to(ui.clip_rect())
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui);
            });
    }

    fn sum_window(&mut self, ui: &mut Ui, state: &mut State) {
        // TopBottomPanel::top(ui.auto_id_with("Pane"))
        //     .show_inside(ui, |ui| {
        //         MenuBar::new()
        //             .ui(ui, |ui| {
        //                 ScrollArea::horizontal()
        //                     .show(ui, |ui| {
        //                         ui.set_height(
        //                             ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
        //                         );
        //                         ui.visuals_mut().button_frame = false;
        //                         if ui.button(RichText::new(X).heading()).clicked() {
        //                             behavior.close = Some(tile_id);
        //                         }
        //                         ui.separator();
        //                         self.top(ui, &mut state)
        //                     })
        //                     .inner
        //             })
        //             .inner
        //     })
        Window::new(format!("{SIGMA} Distance sum"))
            .id(ui.auto_id_with(ID_SOURCE).with("Sum"))
            .constrain_to(ui.clip_rect())
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_sum)
            .show(ui.ctx(), |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                ResetButton::new(&mut state.settings.reset_sum).ui(ui);
                                ui.separator();
                                ResizeButton::new(&mut state.settings.resizable).ui(ui);
                                ui.separator();
                            })
                            .inner
                    })
                    .inner;
                self.sum_central(ui, &mut state.settings)
                // TopBottomPanel::top(ui.auto_id_with("Pane")).show_inside(ui, |ui| {
                // });
                // CentralPanel::default()
                //     // .frame(Frame::central_panel(ui.style()))
                //     .show_inside(ui, |ui| );
            });
    }

    #[instrument(skip_all, err)]
    fn sum_central(&mut self, ui: &mut Ui, settings: &mut Settings) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SumComputed>()
                .get(SumKey::new(&self.calculated, settings))
                .clone()
        });
        Sum::new(&data_frame, settings).show(ui);
        Ok(())
    }
}

pub(crate) mod sum;
pub(crate) mod view;
