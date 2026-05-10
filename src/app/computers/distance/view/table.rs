use crate::{
    app::states::distance::Settings,
    r#const::{ALPHA, DELTA, EQUIVALENT_CHAIN_LENGTH, EUCLIDEAN, FROM, RETENTION_TIME, TO},
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
            ddof: settings.ddof,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Distance format value
type Value = DataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    println!("lazy_frame DD: {}", lazy_frame.clone().collect()?);
    // Filter
    // lazy_frame = lazy_frame.filter(col(FILTER));
    // Compute
    lazy_frame = lazy_frame.with_columns([
        as_struct(vec![
            col(FATTY_ACID)
                .struct_()
                .field_by_name(FROM)
                .fatty_acid()
                .display(),
            col(FATTY_ACID)
                .struct_()
                .field_by_name(TO)
                .fatty_acid()
                .display(),
        ])
        .alias(FATTY_ACID),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(FROM),
                key,
            )
            .alias(FROM),
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(TO),
                key,
            )
            .alias(TO),
            mean_and_standard_deviation_and_array(
                col(RETENTION_TIME).struct_().field_by_name(DELTA),
                key,
            )
            .alias(DELTA),
        ])
        .alias(RETENTION_TIME),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(FROM),
                key,
            )
            .alias(FROM),
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(TO),
                key,
            )
            .alias(TO),
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(DELTA),
                key,
            )
            .alias(DELTA),
        ])
        .alias(EQUIVALENT_CHAIN_LENGTH),
        mean_and_standard_deviation_and_array(col(ALPHA), key),
        mean_and_standard_deviation_and_array(col(EUCLIDEAN), key),
        // col(TEMPERATURE).precision(key.precision, key.significant),
        // as_struct(vec![
        //     col(CHAIN_LENGTH)
        //         .struct_()
        //         .field_by_name(EQUIVALENT_CHAIN_LENGTH)
        //         .precision(key.precision, key.significant),
        //     col(CHAIN_LENGTH)
        //         .struct_()
        //         .field_by_name(FRACTIONAL_CHAIN_LENGTH)
        //         .precision(key.precision, key.significant),
        //     col(CHAIN_LENGTH)
        //         .struct_()
        //         .field_by_name(EQUIVALENT_CARBON_NUMBER)
        //         .precision(key.precision, key.significant),
        // ])
        // .alias(CHAIN_LENGTH),
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
        // as_struct(vec![
        //     col(DERIVATIVE)
        //         .struct_()
        //         .field_by_name(ANGLE)
        //         .precision(key.precision, key.significant),
        //     col(DERIVATIVE)
        //         .struct_()
        //         .field_by_name(SLOPE)
        //         .precision(key.precision, key.significant),
        // ])
        // .alias(DERIVATIVE),
    ]);
    Ok(lazy_frame)
}

fn mean_and_standard_deviation_and_array(expr: Expr, key: Key) -> Expr {
    Array::builder()
        .expr(expr)
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
}
