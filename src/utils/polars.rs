use crate::r#const::EM_DASH;
use polars::prelude::*;
use std::borrow::Cow;

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
