use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::Settings,
    },
    r#const::{
        ABSOLUTE, ADJUSTED, BACKWARD, CHAIN_LENGTH, DEAD_TIME, EQUIVALENT_CARBON_NUMBER,
        EQUIVALENT_CHAIN_LENGTH, FILTER, FORWARD, FRACTIONAL_CHAIN_LENGTH, MASS, RELATIVE,
        RETENTION_FACTOR, RETENTION_TIME, SELECTIVITY_FACTOR, STANDARD, TEMPERATURE,
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
        Array::builder()
            .expr(col(RETENTION_FACTOR))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build(),
        as_struct(vec![
            Array::builder()
                .expr(col(SELECTIVITY_FACTOR).struct_().field_by_name(FORWARD))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
            Array::builder()
                .expr(col(SELECTIVITY_FACTOR).struct_().field_by_name(BACKWARD))
                .ddof(key.ddof)
                .precision(key.precision)
                .significant(key.significant)
                .build(),
        ])
        .alias(SELECTIVITY_FACTOR),
        col(DEAD_TIME).precision(key.precision, key.significant),
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
        Array::builder()
            .expr(col(TEMPERATURE))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build(),
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
        col("_").struct_().with_fields(vec![
            col("_").struct_().with_fields(vec![
                col("_")
                    .struct_()
                    .field_by_name(formatcp!("_{STANDARD}{RETENTION_TIME}"))
                    .arr()
                    .eval(element().precision(key.precision, key.significant), false),
                col("_")
                    .struct_()
                    .field_by_name(formatcp!("_{FORWARD}{RETENTION_TIME}"))
                    .arr()
                    .eval(element().precision(key.precision, key.significant), false),
                col("_")
                    .struct_()
                    .field_by_name(formatcp!("_{BACKWARD}{RETENTION_TIME}"))
                    .arr()
                    .eval(element().precision(key.precision, key.significant), false),
            ]),
            col("_")
                .struct_()
                .field_by_name(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                .struct_()
                .with_fields(vec![
                    col("_")
                        .struct_()
                        .field_by_name(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                        .struct_()
                        .field_by_name(formatcp!("{FORWARD}{RETENTION_TIME}"))
                        .arr()
                        .eval(element().precision(key.precision, key.significant), false),
                    col("_")
                        .struct_()
                        .field_by_name(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                        .struct_()
                        .field_by_name(formatcp!("{BACKWARD}{RETENTION_TIME}"))
                        .arr()
                        .eval(element().precision(key.precision, key.significant), false),
                ]),
        ]),
    ])
}
