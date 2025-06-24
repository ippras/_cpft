use crate::{
    app::states::source::{Filter, Order, Settings, Sort},
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprExt;

const RETENTION_TIME: &str = "RetentionTime";
const ABSOLUTE: &str = "Absolute";
const RELATIVE: &str = "Relative";
const DELTA: &str = "Delta";
const MEAN: &str = "Mean";
const STANDARD_DEVIATION: &str = "StandardDeviation";
const SAMPLE: &str = "Sample";

/// Source display computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source display computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("compute source")
    }
}

/// Source display key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            precision: settings.precision,
            // significant: settings.significant,
            significant: false,
        }
    }
}

/// Source display value
type Value = DataFrame;

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    println!(
        "lazy_frame: {}",
        lazy_frame
            .clone()
            // .select([col(RETENTION_TIME).struct_().field_by_name("*")])
            .collect()?
    );
    lazy_frame = lazy_frame.with_columns([
        col(FATTY_ACID).fatty_acid().format(),
        as_struct(vec![
            as_struct(vec![
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(MEAN)
                    .precision(key.precision, key.significant),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(STANDARD_DEVIATION)
                    .precision(key.precision, key.significant),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name("Values")
                    .list()
                    .eval(element().precision(key.precision, key.significant))
                    .alias(SAMPLE),
            ])
            .alias(ABSOLUTE),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(RELATIVE)
                .precision(key.precision, key.significant),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(DELTA)
                .precision(key.precision, key.significant),
        ])
        .alias(RETENTION_TIME),
    ]);
    Ok(lazy_frame)
}
