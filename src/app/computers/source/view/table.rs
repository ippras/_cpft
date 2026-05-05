use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::Settings,
    },
    r#const::{
        ABSOLUTE, ANGLE, CHAIN_LENGTH, DEAD_TIME, DELTA, DERIVATIVE, EQUIVALENT_CARBON_NUMBER,
        EQUIVALENT_CHAIN_LENGTH, FILTER, FRACTIONAL_CHAIN_LENGTH, MASS, RELATIVE, RETENTION_TIME,
        SLOPE, TEMPERATURE,
    },
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;

/// Source format computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source format computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame.filter(col(FILTER));
        // Format
        lazy_frame = format(lazy_frame, key);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("compute source format")
    }
}

/// Source format key
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

/// Source format value
type Value = DataFrame;

fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        col(FATTY_ACID).fatty_acid().display(),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(ABSOLUTE),
                key,
            )
            .alias(ABSOLUTE),
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(RELATIVE),
                key,
            )
            .alias(RELATIVE),
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(DELTA),
                key,
            )
            .alias(DELTA),
        ])
        .alias(RETENTION_TIME),
        col(DEAD_TIME).precision(key.precision, key.significant),
        mean_and_standard_deviation_and_array(col(TEMPERATURE), key).alias(TEMPERATURE),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(EQUIVALENT_CHAIN_LENGTH),
                key,
            )
            .alias(EQUIVALENT_CHAIN_LENGTH),
            mean_and_standard_deviation_and_array(
                col(CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(FRACTIONAL_CHAIN_LENGTH),
                key,
            )
            .alias(FRACTIONAL_CHAIN_LENGTH),
            col(CHAIN_LENGTH)
                .struct_()
                .field_by_name(EQUIVALENT_CARBON_NUMBER),
        ])
        .alias(CHAIN_LENGTH),
        as_struct(vec![
            col(MASS)
                .struct_()
                .field_by_name("RCO")
                .precision(key.precision, key.significant),
            col(MASS)
                .struct_()
                .field_by_name("RCOO")
                .precision(key.precision, key.significant),
            col(MASS)
                .struct_()
                .field_by_name("RCOOH")
                .precision(key.precision, key.significant),
            col(MASS)
                .struct_()
                .field_by_name("RCOOCH3")
                .precision(key.precision, key.significant),
        ])
        .alias(MASS),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(DERIVATIVE).struct_().field_by_name(ANGLE),
                key,
            )
            .alias(ANGLE),
            mean_and_standard_deviation_and_array(
                col(DERIVATIVE).struct_().field_by_name(SLOPE),
                key,
            )
            .alias(SLOPE),
        ])
        .alias(DERIVATIVE),
    ])
}

fn mean_and_standard_deviation_and_array(expr: Expr, key: Key) -> Expr {
    Array::builder()
        .expr(expr)
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .keep_name(true)
        .build()
}
