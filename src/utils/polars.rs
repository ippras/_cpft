use crate::r#const::{ARRAY, EM_DASH, MEAN, STANDARD_DEVIATION};
use polars::prelude::*;
use polars_ext::expr::ExprExt;
use std::borrow::Cow;
use typed_builder::TypedBuilder;

/// Extension methods for [`Series`]
pub trait SeriesExt {
    fn str_f64(&self, index: usize) -> PolarsResult<Cow<'_, str>>;
}

impl SeriesExt for Series {
    fn str_f64(&self, index: usize) -> PolarsResult<Cow<'_, str>> {
        Ok(self
            .f64()?
            .get(index)
            .filter(|float| !float.is_nan())
            .map_or(Cow::Borrowed(EM_DASH), |float| float.to_string().into()))
    }
}

/// Mean and standard deviation and array
#[derive(TypedBuilder)]
#[builder(build_method(into=Expr))]
pub struct Array {
    expr: Expr,
    #[builder(default)]
    ddof: u8,
    precision: usize,
    #[builder(default)]
    significant: bool,
}

impl From<Array> for Expr {
    fn from(value: Array) -> Self {
        as_struct(vec![
            value
                .expr
                .clone()
                .arr()
                .mean()
                .precision(value.precision, value.significant)
                .alias(MEAN),
            value
                .expr
                .clone()
                .arr()
                .std(value.ddof)
                .precision(value.precision + 1, value.significant)
                .alias(STANDARD_DEVIATION),
            value
                .expr
                .arr()
                .eval(
                    element().precision(value.precision, value.significant),
                    false,
                )
                .alias(ARRAY),
        ])
    }
}
