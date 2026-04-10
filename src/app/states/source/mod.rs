pub(crate) use self::{
    settings::{Axis, Filter, Order, PlotSettings, Settings, Sort, View},
    windows::Windows,
};

use egui::{Context, Id};
use serde::{Deserialize, Serialize};

pub(crate) const ID_SOURCE: &str = "Source";

/// State
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| data.get_persisted_mut_or_insert_with(id, Self::new).clone())
    }

    pub(crate) fn remove(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

mod settings;
mod windows;
