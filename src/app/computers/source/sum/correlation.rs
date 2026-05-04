use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::Settings,
    },
    r#const::*,
    utils::{hash::HashedDataFrame, polars::Array},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::eval_arr;

/// Correlation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Correlation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame
            .filter(col(FATTY_ACID).fatty_acid().is_saturated())
            .select([
                col(MODE),
                col(FATTY_ACID).fatty_acid().carbon().alias(CARBON),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .alias(RETENTION_TIME),
            ]);
        // Group
        lazy_frame = group(lazy_frame)?;
        // Format
        lazy_frame = format(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("Compute source correlation")
    }
}

/// Correlation key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Correlation value
type Value = HashedDataFrame;

/// Group
fn group(lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame
        .group_by_stable([MODE])
        .agg([eval_arr(col(RETENTION_TIME), |element| {
            Ok(pearson_corr(col(CARBON), element))
        })?
        .alias(CORRELATION)]))
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([Array::builder()
        .expr(col(CORRELATION))
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
        .alias(CORRELATION)])
}
