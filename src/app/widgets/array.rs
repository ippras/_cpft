use crate::{
    app::states::source::Settings,
    r#const::{ARRAY, EM_DASH, MEAN, NO_BREAK_SPACE, STANDARD_DEVIATION},
};
use egui::{Color32, Response, TextWrapMode, Ui, WidgetText};
use egui_l20n::prelude::*;
use itertools::Itertools;
use polars::prelude::*;
use polars_utils::format_list;
use typed_builder::TypedBuilder;

/// Mean and standard deviation and array
#[derive(TypedBuilder)]
pub struct Array<'a> {
    series: &'a Series,
    row: usize,

    #[builder(default, setter(strip_option))]
    color: Option<Color32>,
    #[builder(default)]
    mean: bool,
    #[builder(default)]
    standard_deviation: bool,
}

impl Array<'_> {
    pub fn show(&self, ui: &mut Ui) -> PolarsResult<Response> {
        let mean_series = self.series.struct_()?.field_by_name(MEAN)?;
        let mean = mean_series.f64()?.get(self.row);
        let standard_deviation_series = self.series.struct_()?.field_by_name(STANDARD_DEVIATION)?;
        let standard_deviation = standard_deviation_series.f64()?.get(self.row);
        let array_series = self.series.struct_()?.field_by_name(ARRAY)?;
        let array = array_series.array()?.get_as_series(self.row);
        let mut text = if self.mean
            && self.standard_deviation
            && let Some(mean) = mean
            && let Some(standard_deviation) = standard_deviation
        {
            WidgetText::from(format!("{mean}{NO_BREAK_SPACE}±{standard_deviation}"))
        } else if self.mean {
            if let Some(mean) = mean {
                WidgetText::from(mean.to_string())
            } else {
                WidgetText::from(EM_DASH)
            }
        } else if let Some(array) = array {
            WidgetText::from(format!(
                "[{}]",
                array
                    .iter()
                    .format_with(", ", |any_value, f| match any_value {
                        AnyValue::Null => f(&EM_DASH),
                        any_value => f(&any_value),
                    })
            ))
        } else {
            WidgetText::from(EM_DASH)
        };
        if let Some(color) = self.color {
            text = text.color(color);
        }
        let mut response = ui.label(text);
        if response.hovered() {
            // Mean
            if let Some(mean) = mean {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(MEAN));
                    ui.label(mean.to_string());
                });
            }
            // Standard deviation
            if let Some(standard_deviation) = standard_deviation {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(STANDARD_DEVIATION));
                    ui.label(format!("±{standard_deviation}"));
                });
            }
            // Array
            if let Some(sample) = self
                .series
                .struct_()?
                .field_by_name(ARRAY)?
                .array()?
                .get_as_series(self.row)
                && sample.len() > 1
            {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(ARRAY));
                    ui.label(format_list!(sample.iter()));
                });
            }
        }
        Ok(response)
    }
}
