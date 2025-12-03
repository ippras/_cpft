use self::{plot::PlotView, table::TableView};
use crate::{
    app::{
        computers::source::{
            Computed as SourceComputed, Key as SourceKey,
            display::{Computed as DisplayComputed, Key as DisplayKey},
            plot::{Computed as PlotComputed, Key as PlotKey},
        },
        panes::{Behavior, MARGIN, widgets::ViewWidget},
        states::source::{ID_SOURCE, State, View},
    },
    r#const::{MODE, ONSET_TEMPERATURE, TEMPERATURE_STEP},
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use egui::{
    Button, CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea,
    TextStyle, TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, SLIDERS_HORIZONTAL, TABLE, X,
};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::Metadata;
use polars::{
    error::PolarsResult,
    prelude::{ChunkSort, ChunkUnique as _},
};
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

    #[instrument(skip_all, err)]
    fn init(&mut self, ui: &mut Ui, state: &mut State) -> Result<()> {
        self.calculated = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SourceComputed>()
                .get(SourceKey::new(&self.frame.data, &state.settings))
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
        let mut response = ui.heading(TABLE).on_hover_text(ui.localize("source"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.calculated.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        self.reset_button(ui, state);
        ui.separator();
        self.resize_button(ui, state);
        ui.separator();
        self.settings_button(ui, state);
        ui.separator();
        ui.add(ViewWidget::new(&mut state.settings.view));
        ui.separator();
        self.save_button(ui);
        ui.separator();
        self.distance_button(ui, state);
        ui.separator();
        response
    }

    /// Reset button
    fn reset_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.reset_table_state,
            RichText::new(ARROWS_CLOCKWISE).heading(),
        )
        .on_hover_localized("ResetTable");
    }

    /// Resize button
    fn resize_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_localized("ResizeTable");
    }

    /// Settings button
    fn settings_button(&mut self, ui: &mut Ui, state: &mut State) {
        ui.toggle_value(
            &mut state.windows.open_settings,
            RichText::new(SLIDERS_HORIZONTAL).heading(),
        )
        .on_hover_localized("Settings");
    }

    /// Save button
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

    /// Distance button
    fn distance_button(&mut self, ui: &mut Ui, state: &mut State) {
        if ui
            .add_enabled(
                state.settings.view == View::Table,
                Button::new(RichText::new(EXCLUDE).heading()),
            )
            .clicked()
        {
            let data = self.calculated.clone();
            let meta = self.frame.meta.clone();
            let frame = HashedMetaDataFrame::new(meta, data);
            ui.data_mut(|data| data.insert_temp(Id::new("Distance"), frame))
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
                });
                PlotView::new(points, &state.settings).show(ui)
            }
            View::Table => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayComputed>()
                        .get(DisplayKey::new(&self.calculated, &state.settings))
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

pub(crate) mod plot;
pub(crate) mod table;
