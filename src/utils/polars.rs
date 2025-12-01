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
            .map_or(Cow::Borrowed("-"), |float| float.to_string().into()))
    }
}
