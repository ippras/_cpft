use crate::{
    app::{
        MAX_TEMPERATURE,
        states::source::{Filter, Order, Settings, Sort},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;

/// Source computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        // Filter
        lazy_frame = filter(lazy_frame, key)?;
        // Interpolate
        // Sort
        lazy_frame = sort(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("compute source")
    }
}

/// Source key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) filter: &'a Filter,
    pub(crate) logarithmic: bool,
    pub(crate) order: Order,
    pub(crate) relative: &'a Option<FattyAcid>,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            filter: &settings.filter,
            logarithmic: settings.logarithmic,
            order: settings.order,
            relative: &settings.relative,
            sort: settings.sort,
        }
    }
}

/// Source value
type Value = HashedDataFrame;

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    println!("key: {:?}", key.filter);
    println!(
        "COMPUTE0 lazy_frame: {}",
        filter(lazy_frame.clone(), key)?
            .filter(col(FILTER))
            .collect()
            .unwrap()
    );
    lazy_frame = lazy_frame
        .with_columns([
            // Retention time mean
            col(RETENTION_TIME).list().mean().alias("RetentionTimeMean"),
            // Retention time standard deviation
            col(RETENTION_TIME)
                .list()
                .std(key.ddof)
                .alias("RetentionTimeStandardDeviation"),
        ])
        .with_columns([
            // Relative retention time
            relative_time(key)?
                .over([MODE])
                .alias("RelativeRetentionTime"),
            // Delta retention time
            col(FATTY_ACID)
                .fatty_acid()
                .delta(col("RetentionTimeMean"))
                .over([MODE])
                .alias("DeltaRetentionTime"),
            // Temperature
            (col(MODE).struct_().field_by_name(ONSET_TEMPERATURE)
                + col("RetentionTimeMean") * col(MODE).struct_().field_by_name(TEMPERATURE_STEP))
            .clip_max(lit(MAX_TEMPERATURE))
            .alias(TEMPERATURE),
            // FCL
            col(FATTY_ACID)
                .fatty_acid()
                .fractional_chain_length(col("RetentionTimeMean"), key.logarithmic)
                .over([MODE])
                .alias(FRACTIONAL_CHAIN_LENGTH),
            // ECL
            col(FATTY_ACID)
                .fatty_acid()
                .equivalent_chain_length(col("RetentionTimeMean"), key.logarithmic)
                .over([MODE])
                .alias(EQUIVALENT_CHAIN_LENGTH),
            // ECN
            col(FATTY_ACID)
                .fatty_acid()
                .equivalent_carbon_number()
                .alias(EQUIVALENT_CARBON_NUMBER),
        ])
        .with_columns([
            // Slope
            col(FATTY_ACID)
                .fatty_acid()
                .slope(col(EQUIVALENT_CHAIN_LENGTH), col("RetentionTimeMean"))
                .over([MODE])
                .alias(SLOPE),
        ])
        .select([
            col(MODE),
            col(FATTY_ACID),
            // Retention time
            as_struct(vec![
                as_struct(vec![
                    col("RetentionTimeMean").alias(MEAN),
                    col("RetentionTimeStandardDeviation").alias(STANDARD_DEVIATION),
                    col(RETENTION_TIME).alias("Values"),
                ])
                .alias(ABSOLUTE),
                col("RelativeRetentionTime").alias(RELATIVE),
                col("DeltaRetentionTime").alias(DELTA),
            ])
            .alias(RETENTION_TIME),
            // DeadTime
            col("DeadTime"),
            // Temperature
            col(TEMPERATURE),
            // Chain length
            as_struct(vec![
                col(EQUIVALENT_CHAIN_LENGTH),
                col(FRACTIONAL_CHAIN_LENGTH),
                col(EQUIVALENT_CARBON_NUMBER),
            ])
            .alias(CHAIN_LENGTH),
            // Mass
            as_struct(vec![
                col(FATTY_ACID)
                    .fatty_acid()
                    .rco()
                    .relative_atomic_mass(None)
                    .alias("RCO"),
                col(FATTY_ACID)
                    .fatty_acid()
                    .rcoo()
                    .relative_atomic_mass(None)
                    .alias("RCOO"),
                col(FATTY_ACID)
                    .fatty_acid()
                    .rcooh()
                    .relative_atomic_mass(None)
                    .alias("RCOOH"),
                col(FATTY_ACID)
                    .fatty_acid()
                    .rcooch3()
                    .relative_atomic_mass(None)
                    .alias("RCOOCH3"),
            ])
            .alias(MASS),
            // Derivative
            as_struct(vec![col(SLOPE), col(SLOPE).arctan().degrees().alias(ANGLE)])
                .alias(DERIVATIVE),
        ]);
    Ok(lazy_frame)
}

fn relative_time(key: Key) -> PolarsResult<Expr> {
    Ok(match &key.relative {
        Some(fatty_acid) => {
            col("RetentionTimeMean")
                / col("RetentionTimeMean")
                    .filter(
                        col(FATTY_ACID)
                            .fatty_acid()
                            .equal(FattyAcidExpr::try_from(fatty_acid)?),
                    )
                    .first()
        }
        None => lit(f64::NAN),
    })
}

fn filter(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut expr = lit(true);
    for &onset_temperature in &key.filter.onset_temperatures {
        expr = expr.and(
            col(MODE)
                .struct_()
                .field_by_name(ONSET_TEMPERATURE)
                .neq(onset_temperature),
        );
    }
    for &temperature_step in &key.filter.temperature_steps {
        expr = expr.and(
            col(MODE)
                .struct_()
                .field_by_name(TEMPERATURE_STEP)
                .neq(temperature_step),
        );
    }
    for fatty_acid in &key.filter.fatty_acids {
        expr = expr.and(
            col(FATTY_ACID)
                .fatty_acid()
                .equal(FattyAcidExpr::try_from(fatty_acid)?)
                .not(),
        );
    }
    lazy_frame = lazy_frame.with_column(expr.alias(FILTER));
    Ok(lazy_frame)
}

fn sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let sort_options = SortMultipleOptions::new()
        .with_nulls_last(true)
        .with_order_descending(key.order == Order::Descending);
    match key.sort {
        Sort::FattyAcid => lazy_frame.sort_by_exprs(
            [
                col(MODE),
                col(FATTY_ACID).fatty_acid().carbon(),
                col(FATTY_ACID).fatty_acid().unsaturation(),
                col(FATTY_ACID).fatty_acid().indices(),
            ],
            sort_options,
        ),
        Sort::RetentionTime => lazy_frame.sort([MODE], sort_options.clone()).select([all()
            .as_expr()
            .sort_by(
                &[col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(MEAN)],
                sort_options,
            )
            .over([col(MODE)])]),
    }
}

/// Saturated
pub trait Saturated {
    /// Delta
    fn delta(self, expr: Expr) -> Expr;

    /// Slope
    fn slope(self, dividend: Expr, divisor: Expr) -> Expr;

    /// Backward saturated
    fn backward(self, expr: Expr) -> Expr;

    /// Forward saturated
    fn forward(self, expr: Expr) -> Expr;
}

impl Saturated for FattyAcidExpr {
    fn delta(self, expr: Expr) -> Expr {
        self.clone().backward(expr.clone()) - self.clone().forward(expr)
    }

    fn slope(self, dividend: Expr, divisor: Expr) -> Expr {
        self.clone().delta(dividend) / self.clone().delta(divisor)
    }

    fn backward(self, expr: Expr) -> Expr {
        ternary_expr(self.is_saturated(), expr, lit(NULL))
            .fill_null_with_strategy(FillNullStrategy::Backward(None))
    }

    fn forward(self, expr: Expr) -> Expr {
        ternary_expr(self.is_saturated(), expr, lit(NULL))
            .fill_null_with_strategy(FillNullStrategy::Forward(None))
    }
}

pub(crate) mod display;
pub(crate) mod plot;
