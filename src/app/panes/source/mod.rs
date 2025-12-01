use self::table::TableView;
use super::{Behavior, MARGIN, widgets::ViewWidget};
use crate::{
    app::{
        computers::source::{
            Computed as SourceComputed,
            Key as SourceKey,
            display::{Computed as DisplayComputed, Key as DisplayKey},
            // plot::{Key as PlotKey, Computed as PlotComputed},
        },
        states::source::{ID_SOURCE, State, View},
    },
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use egui::{
    Button, CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea,
    TextStyle, TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, SLIDERS_HORIZONTAL, TABLE, X,
};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, from_fn};
use tracing::instrument;

/// Source pane
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
                .cache::<SourceComputed>()
                .get(SourceKey::new(&self.frame.data, &state.settings))
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
        let mut response = ui.heading(TABLE).on_hover_text(ui.localize("source"));
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
        self.reset_button(ui, state);
        ui.separator();
        // Resize
        self.resize_button(ui, state);
        ui.separator();
        // Settings
        self.settings_button(ui, state);
        ui.separator();
        // View
        ui.add(ViewWidget::new(&mut state.settings.view));
        ui.separator();
        // Distance
        self.distance_button(ui, state);
        ui.separator();
        // Save
        self.save_button(ui);
        ui.separator();
        response
    }

    // Reset button
    fn reset_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.reset_table_state,
            RichText::new(ARROWS_CLOCKWISE).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResetTable"));
        });
    }

    // Resize button
    fn resize_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResizeTable"));
        });
    }

    /// Settings button
    fn settings_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.windows.open_settings,
            RichText::new(SLIDERS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Settings"));
        });
    }

    /// Distance button
    fn distance_button(&mut self, ui: &mut Ui, state: &mut State) {
        if ui
            .add_enabled(
                state.settings.view == View::Table,
                Button::new(RichText::new(EXCLUDE).heading()),
            )
            .clicked()
        {
            let data = self.calculate(ui, state);
            let meta = self.frame.meta.clone();
            let frame = HashedMetaDataFrame::new(meta, data);
            ui.data_mut(|data| data.insert_temp(Id::new("Distance"), frame))
        }
    }

    fn save_button(&self, ui: &mut Ui) {
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
                let _ = self.save_ron(&name, meta);
            }
        });
    }

    #[instrument(skip(self), err)]
    fn save_ron(&self, name: impl Debug + Display, meta: &Metadata) -> Result<()> {
        // let data = self
        //     .target
        //     .data_frame
        //     .clone()
        //     .lazy()
        //     .select([
        //         col(LABEL),
        //         col(FATTY_ACID),
        //         col(STEREOSPECIFIC_NUMBERS123),
        //         col(STEREOSPECIFIC_NUMBERS13),
        //         col(STEREOSPECIFIC_NUMBERS2),
        //     ])
        //     .collect()?;
        // let frame = MetaDataFrame::new(meta, data);
        // ron::save(&frame, &format!("{name}.fa.utca.ron"))?;
        Ok(())
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        let frame = self.calculate(ui, state);
        match state.settings.view {
            View::Plot => {
                // let points = ui.memory_mut(|memory| {
                //     memory
                //         .caches
                //         .cache::<SourcePlotComputed>()
                //         .get(SourcePlotKey {
                //             data_frame: &self.target,
                //             settings: &state.settings,
                //         })
                // });
                // PlotView::new(points, &state.settings).show(ui)
            }
            View::Table => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayComputed>()
                        .get(DisplayKey::new(&frame, &state.settings))
                });
                TableView::new(&data_frame, state).show(ui)
            }
        };
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Source settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                let _ = state.settings.show(ui);
            });
    }
}

// mod plot;
mod table;
