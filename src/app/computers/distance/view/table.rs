use crate::{
    app::{
        computers::{distance::process::OUTPUT_SCHEMA as INPUT_SCHEMA, matches_schema},
        states::distance::Settings,
    },
    r#const::{
        DEAD_TIME, DISTANCE, EQUIVALENT_CHAIN_LENGTH, FROM, RETENTION_TIME, SELECTIVITY_FACTOR, TO,
    },
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;

/// Distance format computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance format computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = format(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("Compute distance format")
    }
}

/// Distance format key
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
            ddof: settings.mean_and_standard_deviation.ddof,
            precision: settings.precision_and_significant.precision,
            significant: settings.precision_and_significant.significant,
        }
    }
}

/// Distance format value
type Value = DataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Filter
    // lazy_frame = lazy_frame.filter(col(FILTER));
    // Compute
    lazy_frame = lazy_frame.with_columns([
        as_struct(vec![
            col(FATTY_ACID)
                .struct_()
                .field_by_name(FROM)
                .fatty_acid()
                .delta(),
            col(FATTY_ACID)
                .struct_()
                .field_by_name(TO)
                .fatty_acid()
                .delta(),
        ])
        .alias(FATTY_ACID),
        col(DEAD_TIME).precision(key.precision, key.significant),
        format_struct(col(RETENTION_TIME), key),
        format_struct(col(EQUIVALENT_CHAIN_LENGTH), key),
        format_array(col(SELECTIVITY_FACTOR), key),
        // col(TEMPERATURE).precision(key.precision, key.significant),
        // as_struct(vec![
        //     col(MASS)
        //         .struct_()
        //         .field_by_name("RCO")
        //         .precision(key.precision, key.significant),
        //     col(MASS)
        //         .struct_()
        //         .field_by_name("RCOO")
        //         .precision(key.precision, key.significant),
        //     col(MASS)
        //         .struct_()
        //         .field_by_name("RCOOH")
        //         .precision(key.precision, key.significant),
        //     col(MASS)
        //         .struct_()
        //         .field_by_name("RCOOCH3")
        //         .precision(key.precision, key.significant),
        // ])
        // .alias(MASS),
    ]);
    Ok(lazy_frame)
}

fn format_struct(expr: Expr, key: Key) -> Expr {
    as_struct(vec![
        format_array(expr.clone().struct_().field_by_name(FROM), key),
        format_array(expr.clone().struct_().field_by_name(TO), key),
        format_array(expr.clone().struct_().field_by_name(DISTANCE), key),
    ])
    .name()
    .keep()
}

fn format_array(expr: Expr, key: Key) -> Expr {
    Array::builder()
        .expr(expr)
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
}
