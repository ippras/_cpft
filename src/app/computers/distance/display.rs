use crate::{
    app::states::distance::Settings,
    r#const::{
        ABSOLUTE, ARRAY, CHAIN_LENGTH, DELTA, EQUIVALENT_CARBON_NUMBER, EQUIVALENT_CHAIN_LENGTH,
        FRACTIONAL_CHAIN_LENGTH, MEAN, RELATIVE, RETENTION_TIME, STANDARD_DEVIATION, TEMPERATURE,
        *,
    },
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprExt;

/// Distance display computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance display computer
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
        self.try_compute(key).expect("compute distance display")
    }
}

/// Distance display key
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

/// Distance display value
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
                .format(),
            col(FATTY_ACID)
                .struct_()
                .field_by_name(TO)
                .fatty_acid()
                .format(),
        ])
        .alias(FATTY_ACID),
        as_struct(vec![
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(FROM)
                .precision(key.precision, key.significant),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(TO)
                .precision(key.precision, key.significant),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(DELTA)
                .precision(key.precision, key.significant),
        ])
        .alias(RETENTION_TIME),
        as_struct(vec![
            col(EQUIVALENT_CHAIN_LENGTH)
                .struct_()
                .field_by_name(FROM)
                .precision(key.precision, key.significant),
            col(EQUIVALENT_CHAIN_LENGTH)
                .struct_()
                .field_by_name(TO)
                .precision(key.precision, key.significant),
            col(EQUIVALENT_CHAIN_LENGTH)
                .struct_()
                .field_by_name(DELTA)
                .precision(key.precision, key.significant),
        ])
        .alias(EQUIVALENT_CHAIN_LENGTH),
        col(ALPHA).precision(key.precision, key.significant),
        col(EUCLIDEAN).precision(key.precision, key.significant),
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
