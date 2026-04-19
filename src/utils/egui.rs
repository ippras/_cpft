use crate::r#const::EM_DASH;
use egui::WidgetText;
use std::fmt::Display;

/// To [`WidgetText`]
pub trait ToWidgetText {
    fn to_widget_text(&self) -> WidgetText;
}

impl<T: Display> ToWidgetText for Option<T> {
    fn to_widget_text(&self) -> WidgetText {
        match self {
            None => WidgetText::from(EM_DASH),
            Some(t) => WidgetText::from(t.to_string()),
        }
    }
}
