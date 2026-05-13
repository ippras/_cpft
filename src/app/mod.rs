use self::{
    panes::{Behavior, Pane},
    states::State,
};
use crate::{
    app::widgets::buttons::{
        GridButton, HorizontalButton, ReactiveButton, ResetButton, SettingsButton, TabsButton,
        VerticalButton,
    },
    localization::ContextExt as _,
    presets::AGILENT,
};
use anyhow::Result;
use data::Data;
use eframe::{APP_KEY, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, FontDefinitions, Frame, Id, LayerId, Layout,
    MenuBar, Order, Panel, RichText, ScrollArea, Sides, TextStyle, TopBottomPanel, Ui, Widget,
    Window, warn_if_debug_build,
};
use egui_ext::{HoveredFileExt, LightDarkButton};
use egui_l20n::prelude::*;
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        DATABASE, GRID_FOUR, SLIDERS_HORIZONTAL, SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL,
        TABS,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{TreeExt as _, VERTICAL};
use metadata::egui::MetadataWidget;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, str, time::Duration};
use tracing::{error, info};

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const _NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);
const ICON_SIZE: f32 = 32.0;
const ID_SOURCE: &str = "CPFT";
const MAX_TEMPERATURE: f64 = 250.0;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    // Data
    data: Data,
    // Panes
    tree: Tree<Pane>,
    behavior: Behavior,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: Data::default(),
            tree: Tree::empty("Tree"),
            behavior: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_localizations();

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }

    fn drag_and_drop(&mut self, ui: &Ui) {
        // Preview hovering files
        if let Some(text) = ui.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ui.layer_painter(LayerId::new(Order::Foreground, Id::new("FileDropTarget")));
            let content_rect = ui.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ui.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ui.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped_file in dropped_files {
                if let Err(error) = || -> Result<()> {
                    // let frame = MetaDataFrame::read(Cursor::new(dropped_file.bytes()?))?;
                    // trace!(?frame);
                    // self.data.stack(&frame.data)?;
                    // ctx.request_repaint();
                    Ok(())
                }() {
                    error!(%error);
                }
                // match ron(&dropped_file) {
                //     Ok(data_frame) => {
                //         trace!(?data_frame);
                //         self.data.stack(&data_frame).unwrap();
                //         if !self.tree.tiles.is_empty() {
                //             self.tree = Tree::empty("tree");
                //         }
                //         // self.tree
                //         //     .insert_pane(Pane::source(self.data.data_frame.clone()));
                //         // self.tree
                //         //     .insert_pane(Pane::distance(self.data.data_frame.clone()));
                //         trace!(?self.data);
                //     }
                //     Err(error) => {
                //         error!(%error);
                //         // self.toasts
                //         //     .error(format!("{}: {error}", dropped.display()))
                //         //     .set_closable(true)
                //         //     .set_duration(Some(NOTIFICATIONS_DURATION));
                //         continue;
                //     }
                // };
            }
            // TODO
            // println!("data_frame: {}", self.data.data_frame);
            // let data_frame = self
            //     .data
            //     .data_frame
            //     .clone()
            //     .lazy()
            //     .select([
            //         as_struct(vec![
            //             col("OnsetTemperature").alias("OnsetTemperature"),
            //             col("TemperatureStep").alias("TemperatureStep"),
            //         ])
            //         .alias("Mode"),
            //         col("FA"),
            //         col("Time"),
            //     ])
            //     .cache()
            //     .sort(["Mode"], SortMultipleOptions::new())
            //     .select([all()
            //         .sort_by(&[col("Time").list().mean()], SortMultipleOptions::new())
            //         .over([MODE])])
            //     .collect()
            //     .unwrap();
            // println!("data_frame: {data_frame}");
            // self.tree
            //     .insert_pane::<VERTICAL>(Pane::source(data_frame.clone()));
            // self.tree
            //     .insert_pane::<VERTICAL>(Pane::distance(data_frame.clone()));
            // data::save("data_frame.bin", data::Format::Bin, data_frame).unwrap();
        }
    }
}

impl App {
    fn panels(&mut self, ui: &mut Ui, state: &mut State) {
        self.top_panel(ui, state);
        self.bottom_panel(ui);
        self.central_panel(ui);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ui: &mut Ui) {
        Panel::bottom("BottomPanel").show_inside(ui, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Sides::new().show(
                    ui,
                    |_| {},
                    |ui| {
                        warn_if_debug_build(ui);
                        ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                        ui.separator();
                    },
                );
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ui: &mut Ui) {
        CentralPanel::default()
            .frame(Frame::central_panel(&ui.style()).inner_margin(0))
            .show_inside(ui, |ui| {
                let mut behavior = Behavior { close: None };
                self.tree.ui(&mut behavior, ui);
                if let Some(id) = behavior.close {
                    self.tree.tiles.remove(id);
                }
            });
    }

    // Top panel
    fn top_panel(&mut self, ui: &mut Ui, state: &mut State) {
        Panel::top("TopPanel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    ReactiveButton::new(&mut state.settings.reactive)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Light/Dark
                    ui.light_dark_button(ICON_SIZE);
                    ui.separator();
                    ResetButton::new(&mut state.settings.reset_state)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    self.layouts(ui, state);
                    ui.separator();
                    SettingsButton::new(&mut state.windows.open_settings)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Database
                    self.database_button(ui);
                    ui.separator();
                });
            });
        });
    }

    fn layouts(&mut self, ui: &mut Ui, state: &mut State) {
        VerticalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        HorizontalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        GridButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        TabsButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
    }

    /// Database button
    fn database_button(&mut self, ui: &mut Ui) {
        ui.menu_button(RichText::new(DATABASE).size(ICON_SIZE), |ui| {
            let mut response = ui.button(
                RichText::new(format!("{DATABASE} {}", AGILENT.meta.format(" "))).heading(),
            );
            response = response.on_hover_ui(|ui| {
                MetadataWidget::new(&AGILENT.meta).show(ui);
            });
            if response.clicked() {
                self.tree
                    .insert_pane::<VERTICAL>(Pane::source(AGILENT.clone()));
            }
        })
        .response
        .on_hover_localized("Database");
    }
}

// Windows
impl App {
    fn windows(&mut self, ui: &Ui, state: &mut State) {
        // self.about_window(ctx, state);
        self.settings_window(ui, state);
    }

    // fn about_window(&mut self, ctx: &Context, state: &mut State) {
    //     Window::new(format!("{INFO} About"))
    //         .open(&mut state.windows.open_about)
    //         .show(ctx, |ui| About.ui(ui));
    // }

    fn settings_window(&mut self, ui: &Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Settings"))
            .open(&mut state.windows.open_settings)
            .show(ui, |ui| {
                state.settings.show(ui);
            });
    }
}

impl App {
    fn distance(&mut self, ui: &Ui) {
        if let Some(frame) = ui.data_mut(|data| data.remove_temp(Id::new("Distance"))) {
            self.tree.insert_pane::<VERTICAL>(Pane::distance(frame));
        }
    }

    fn state(&mut self, ui: &Ui, state: &mut State) {
        if state.settings.reset_state {
            *self = Default::default();
            // Cache
            let caches = ui.memory_mut(|memory| memory.caches.clone());
            ui.memory_mut(|memory| {
                memory.caches = caches;
            });
            ui.set_localizations();
            state.settings.reset_state = false;
        }
        if let Some(container_kind) = state.settings.layout.container_kind.take()
            && let Some(id) = self.tree.root
            && let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id)
        {
            container.set_kind(container_kind);
        }
        if state.settings.reactive {
            ui.request_repaint();
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let mut state = State::load(ui, Id::new(ID_SOURCE));
        self.distance(ui);
        // Pre update
        self.panels(ui, &mut state);
        self.windows(ui, &mut state);
        // Post update
        self.drag_and_drop(ui);
        self.state(ui, &mut state);
        state.store(ui, Id::new(ID_SOURCE));
    }
}

mod computers;
mod data;
mod panes;
mod states;
mod widgets;
