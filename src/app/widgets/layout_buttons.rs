use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{GRID_FOUR, SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL, TABS};
use egui_tiles::ContainerKind;

/// Vertical button widget
#[derive(Debug)]
pub struct VerticalButton<'a> {
    current_value: &'a mut Option<ContainerKind>,
    size: Option<f32>,
}

impl<'a> VerticalButton<'a> {
    pub fn new(current_value: &'a mut Option<ContainerKind>) -> Self {
        Self {
            current_value,
            size: None,
        }
    }

    pub fn with_size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for VerticalButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(SQUARE_SPLIT_VERTICAL);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.selectable_value(
            self.current_value,
            Some(ContainerKind::Vertical),
            ui.localize("Vertical"),
        )
        .on_hover_localized("Vertical.hover")
    }
}

/// Horizontal button widget
#[derive(Debug)]
pub struct HorizontalButton<'a> {
    current_value: &'a mut Option<ContainerKind>,
    size: Option<f32>,
}

impl<'a> HorizontalButton<'a> {
    pub fn new(current_value: &'a mut Option<ContainerKind>) -> Self {
        Self {
            current_value,
            size: None,
        }
    }

    pub fn with_size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for HorizontalButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(SQUARE_SPLIT_HORIZONTAL);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.selectable_value(
            self.current_value,
            Some(ContainerKind::Horizontal),
            ui.localize("Horizontal"),
        )
        .on_hover_localized("Horizontal.hover")
    }
}

/// Grid button widget
#[derive(Debug)]
pub struct GridButton<'a> {
    current_value: &'a mut Option<ContainerKind>,
    size: Option<f32>,
}

impl<'a> GridButton<'a> {
    pub fn new(current_value: &'a mut Option<ContainerKind>) -> Self {
        Self {
            current_value,
            size: None,
        }
    }

    pub fn with_size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for GridButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(GRID_FOUR);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.selectable_value(
            self.current_value,
            Some(ContainerKind::Grid),
            ui.localize("Grid"),
        )
        .on_hover_localized("Grid.hover")
    }
}

/// Tabs button widget
#[derive(Debug)]
pub struct TabsButton<'a> {
    current_value: &'a mut Option<ContainerKind>,
    size: Option<f32>,
}

impl<'a> TabsButton<'a> {
    pub fn new(current_value: &'a mut Option<ContainerKind>) -> Self {
        Self {
            current_value,
            size: None,
        }
    }

    pub fn with_size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for TabsButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(TABS);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.selectable_value(
            self.current_value,
            Some(ContainerKind::Tabs),
            ui.localize("Tabs"),
        )
        .on_hover_localized("Tabs.hover")
    }
}
