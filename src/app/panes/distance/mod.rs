use self::{plot::PlotView, sum::Sum, table::TableView};
use crate::{
    app::{
        computers::distance::{
            Computed as DistanceComputed, Key as DistanceKey,
            display::{Computed as DisplayComputed, Key as DisplayKey},
            plot::{Computed as PlotComputed, Key as PlotKey},
            sum::{Computed as SumComputed, Key as SumKey},
        },
        panes::{
            Behavior, MARGIN,
            widgets::{ResetButton, ResizeButton, SettingsButton, ViewButton},
        },
        states::distance::{Settings, State, View},
    },
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Widget as _, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{EXCLUDE, FLOPPY_DISK, SIGMA, SLIDERS_HORIZONTAL, X};
use egui_tiles::{TileId, UiResponse};
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
        let response = TopBottomPanel::top(ui.auto_id_with("Pane"))
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
                .cache::<DistanceComputed>()
                .get(DistanceKey::new(&self.frame.data, &state.settings))
        });
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui.heading(EXCLUDE).on_hover_localized("Distance");
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.calculated.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        ResetButton::new(&mut state.settings.reset_table).ui(ui);
        ui.separator();
        ResizeButton::new(&mut state.settings.resizable).ui(ui);
        ui.separator();
        SettingsButton::new(&mut state.windows.open_settings).ui(ui);
        ui.separator();
        ViewButton::new(&mut state.settings.view).ui(ui);
        ui.separator();
        self.save_button(ui);
        ui.separator();
        self.sum_button(ui, state);
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
                });
                PlotView::new(points, &state.settings).show(ui)
            }
            View::Table => {
                // Display
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayComputed>()
                        .get(DisplayKey::new(&self.calculated, &state.settings))
                });
                TableView::new(&data_frame, &mut state.settings).show(ui)
            }
        };
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
        self.sum_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Distance settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui);
            });
    }

    fn sum_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SIGMA} Distance sum"))
            .id(ui.auto_id_with(ID_SOURCE).with("Sum"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_sum)
            .show(ui.ctx(), |ui| self.sum_content(ui, &mut state.settings));
    }

    #[instrument(skip_all, err)]
    fn sum_content(&mut self, ui: &mut Ui, settings: &mut Settings) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SumComputed>()
                .get(SumKey::new(&self.calculated, settings))
        });
        Sum::new(&data_frame, settings).show(ui);
        Ok(())
    }
}

pub(crate) mod plot;
pub(crate) mod sum;
pub(crate) mod table;
