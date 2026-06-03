use self::{settings::Settings, windows::Windows};
use egui::{Id, Ui};
use serde::{Deserialize, Serialize};

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
    pub(crate) fn load(ui: &Ui, id: Id) -> Self {
        ui.data_mut(|data| data.get_persisted_mut_or_insert_with(id, Self::new).clone())
    }

    pub(crate) fn store(self, ui: &Ui, id: Id) {
        ui.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

pub(crate) mod distance;
pub(crate) mod source;

mod settings;
mod windows;
