use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::Settings,
    },
    r#const::{
        ABSOLUTE, ADJUSTED, ANGLE, CHAIN_LENGTH, DEAD_TIME, DERIVATIVE, EQUIVALENT_CARBON_NUMBER,
        EQUIVALENT_CHAIN_LENGTH, FILTER, FRACTIONAL_CHAIN_LENGTH, MASS, RELATIVE, RETENTION_TIME,
        SLOPE, TEMPERATURE,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
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
            Array::builder()
                .expr(col(RETENTION_TIME).struct_().field_by_name(ABSOLUTE))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
            Array::builder()
                .expr(col(RETENTION_TIME).struct_().field_by_name(RELATIVE))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
            Array::builder()
                .expr(col(RETENTION_TIME).struct_().field_by_name(ADJUSTED))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
        ])
        .alias(RETENTION_TIME),
        col(DEAD_TIME).precision(key.precision, key.significant),
        Array::builder()
            .expr(col(TEMPERATURE))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build(),
        as_struct(vec![
            Array::builder()
                .expr(
                    col(CHAIN_LENGTH)
                        .struct_()
                        .field_by_name(EQUIVALENT_CHAIN_LENGTH),
                )
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
            Array::builder()
                .expr(
                    col(CHAIN_LENGTH)
                        .struct_()
                        .field_by_name(FRACTIONAL_CHAIN_LENGTH),
                )
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
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
            Array::builder()
                .expr(col(DERIVATIVE).struct_().field_by_name(ANGLE))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
            Array::builder()
                .expr(col(DERIVATIVE).struct_().field_by_name(SLOPE))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
        ])
        .alias(DERIVATIVE),
        as_struct(vec![{
            col("_")
                .struct_()
                .field_by_name(formatcp!("_{RELATIVE}{RETENTION_TIME}"))
                .arr()
                .eval(element().precision(key.precision, key.significant), false)
        }])
        .alias("_"),
    ])
}
