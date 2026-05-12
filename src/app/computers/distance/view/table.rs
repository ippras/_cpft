use crate::{
    app::states::distance::Settings,
    r#const::{SELECTIVITY_FACTOR, DELTA, EQUIVALENT_CHAIN_LENGTH, FROM, RETENTION_TIME, TO},
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
        format_struct(RETENTION_TIME, key),
        format_struct(EQUIVALENT_CHAIN_LENGTH, key),
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

fn format_struct(name: &str, key: Key) -> Expr {
    as_struct(vec![
        format_array(col(name).struct_().field_by_name(FROM), key),
        format_array(col(name).struct_().field_by_name(TO), key),
        format_array(col(name).struct_().field_by_name(DELTA), key),
    ])
    .alias(name)
}

fn format_array(expr: Expr, key: Key) -> Expr {
    Array::builder()
        .expr(expr)
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
}
