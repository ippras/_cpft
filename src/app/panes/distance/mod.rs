// use self::{plot::PlotView, table::TableView};
use self::table::TableView;
use crate::{
    app::{
        computers::distance::{
            Computed as DistanceComputed,
            Key as DistanceKey,
            filtered::{Computed as DistanceFilteredComputed, Key as DistanceFilteredKey},
            // plot::{Computed as DistancePlotComputed, Key as DistancePlotKey},
        },
        panes::{Behavior, MARGIN},
        states::distance::{State, View},
    },
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, GEAR, SLIDERS_HORIZONTAL, X,
};
use egui_tiles::{TileId, UiResponse};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, from_fn};
use tracing::error;

const ID_SOURCE: &str = "Distance";

/// Distance pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frame: HashedMetaDataFrame,
}

impl Pane {
    pub(crate) fn new(frame: HashedMetaDataFrame) -> Self {
        Self { id: None, frame }
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

    fn calculate(&self, ui: &mut Ui, state: &mut State) -> HashedDataFrame {
        ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DistanceComputed>()
                .get(DistanceKey::new(&self.frame.data, &state.settings))
        })
    }

    fn filter(&self, ui: &mut Ui, state: &mut State) -> HashedDataFrame {
        let frame = self.calculate(ui, state);
        ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<DistanceFilteredComputed>()
                .get(DistanceFilteredKey::new(&frame, &state.settings))
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

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui.heading(EXCLUDE).on_hover_ui(|ui| {
            ui.label(ui.localize("Distance"));
        });
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!(
                "{}/{:x}",
                self.id(),
                self.calculate(ui, state).hash
            ))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .clicked()
        {
            state.reset_table_state = true;
        }
        ui.separator();
        // Resize
        ui.toggle_value(
            &mut state.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(ui.localize("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut state.windows.open_settings,
            RichText::new(GEAR).heading(),
        );
        ui.separator();
        // // View
        // ui.add(ViewWidget::new(&mut state.settings.view));
        // ui.separator();
        // // Save
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
        ui.separator();
        response
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        // Distance
        let frame = self.filter(ui, state);
        // // Filtered
        // let data_frame = ui.memory_mut(|memory| {
        //     memory
        //         .caches
        //         .cache::<DistanceFilteredComputed>()
        //         .get(DistanceFilteredKey::new(&frame, &state.settings))
        // });
        match state.settings.view {
            View::Plot => {
                // let points = ui.memory_mut(|memory| {
                //     memory
                //         .caches
                //         .cache::<DistancePlotComputed>()
                //         .get(DistancePlotKey {
                //             data_frame: &data_frame,
                //             settings: &state.settings,
                //         })
                // });
                // PlotView::new(points, &state.settings.plot).show(ui)
            }
            View::Table => TableView::new(&frame.data_frame, state).show(ui),
        };
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Distance settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui, &self.frame.data);
            });
    }
}

// mod plot;
mod table;
