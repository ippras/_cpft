pub(crate) use self::{distance::Pane as DistancePane, source::Pane as SourcePane};

use crate::utils::hash::HashedMetaDataFrame;
use egui::{Ui, Vec2, WidgetText, vec2};
use egui_tiles::{TileId, UiResponse};
use serde::{Deserialize, Serialize};

const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Pane
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Pane {
    Source(SourcePane),
    Distance(DistancePane),
}

impl Pane {
    pub(crate) fn source(frame: HashedMetaDataFrame) -> Self {
        Self::Source(SourcePane::new(frame))
    }

    pub(crate) fn distance(frame: HashedMetaDataFrame) -> Self {
        Self::Distance(DistancePane::new(frame))
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Source(_) => "Source",
            Self::Distance(_) => "Distance",
        }
    }
}

/// Behavior
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().to_string().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        match pane {
            Pane::Source(pane) => pane.ui(ui, self, tile_id),
            Pane::Distance(pane) => pane.ui(ui, self, tile_id),
        }
    }
}

pub(crate) mod distance;
pub(crate) mod source;
pub(crate) mod widgets;
