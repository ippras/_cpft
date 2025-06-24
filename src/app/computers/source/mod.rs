use crate::{
    app::{
        MAX_TEMPERATURE,
        states::source::{Filter, Order, Settings, Sort},
    },
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;

const RETENTION_TIME: &str = "RetentionTime";

/// Source computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key)?;
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
                .over(["Mode"])
                .alias("RelativeRetentionTime"),
            // Delta retention time
            col(FATTY_ACID)
                .fatty_acid()
                .delta(col("RetentionTimeMean"))
                .over(["Mode"])
                .alias("DeltaRetentionTime"),
            // Temperature
            (col("Mode").struct_().field_by_name("OnsetTemperature")
                + col("RetentionTimeMean")
                    * col("Mode").struct_().field_by_name("TemperatureStep"))
            .clip_max(lit(MAX_TEMPERATURE))
            .alias("Temperature"),
            // FCL
            col(FATTY_ACID)
                .fatty_acid()
                .fractional_chain_length(col("RetentionTimeMean"), key.logarithmic)
                .over(["Mode"])
                .alias("FCL"),
            // ECL
            col(FATTY_ACID)
                .fatty_acid()
                .equivalent_chain_length(col("RetentionTimeMean"), key.logarithmic)
                .over(["Mode"])
                .alias("EquivalentChainLength"),
            // ECN
            col(FATTY_ACID)
                .fatty_acid()
                .equivalent_carbon_number()
                .alias("ECN"),
        ])
        .with_columns([
            // Slope
            col(FATTY_ACID)
                .fatty_acid()
                .slope(col("EquivalentChainLength"), col("RetentionTimeMean"))
                .over(["Mode"])
                .alias("Slope"),
        ])
        .select([
            col("Mode"),
            col(FATTY_ACID),
            // Retention time
            as_struct(vec![
                as_struct(vec![
                    col("RetentionTimeMean").alias("Mean"),
                    col("RetentionTimeStandardDeviation").alias("StandardDeviation"),
                    col(RETENTION_TIME).alias("Values"),
                ])
                .alias("Absolute"),
                col("RelativeRetentionTime").alias("Relative"),
                col("DeltaRetentionTime").alias("Delta"),
            ])
            .alias(RETENTION_TIME),
            // DeadTime
            col("DeadTime"),
            // Temperature
            col("Temperature"),
            // Chain length
            as_struct(vec![col("EquivalentChainLength"), col("FCL"), col("ECN")])
                .alias("ChainLength"),
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
            .alias("Mass"),
            // Derivative
            as_struct(vec![
                col("Slope"),
                col("Slope").arctan().degrees().alias("Angle"),
            ])
            .alias("Derivative"),
        ]);
    // Filter
    if let Some(predicate) = filter(&key.filter)? {
        lazy_frame = lazy_frame.filter(predicate);
    }
    // Interpolate
    // Sort
    lazy_frame = sort(lazy_frame, key);
    Ok(lazy_frame)
}

fn filter(filter: &Filter) -> PolarsResult<Option<Expr>> {
    let mut expr = None;
    if !filter.onset_temperatures.is_empty() {
        for &onset_temperature in &filter.onset_temperatures {
            expr = Some(
                expr.unwrap_or(lit(true)).and(
                    col("Mode")
                        .struct_()
                        .field_by_name("OnsetTemperature")
                        .neq(onset_temperature),
                ),
            );
        }
    }
    if !filter.temperature_steps.is_empty() {
        for &temperature_step in &filter.temperature_steps {
            expr = Some(
                expr.unwrap_or(lit(true)).and(
                    col("Mode")
                        .struct_()
                        .field_by_name("TemperatureStep")
                        .neq(temperature_step),
                ),
            );
        }
    }
    if !filter.fatty_acids.is_empty() {
        for fatty_acid in &filter.fatty_acids {
            expr = Some(
                expr.unwrap_or(lit(true)).and(
                    col(FATTY_ACID)
                        .fatty_acid()
                        .equal(FattyAcidExpr::try_from(fatty_acid)?)
                        .not(),
                ),
            );
        }
    }
    Ok(expr)
}

fn sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if key.order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    match key.sort {
        Sort::FattyAcid => lazy_frame.sort_by_exprs([col("Mode"), col(FATTY_ACID)], sort_options),
        Sort::Time => lazy_frame
            .sort(["Mode"], sort_options.clone())
            .select([all()
                .as_expr()
                .sort_by(
                    &[
                        col("ChainLength")
                            .struct_()
                            .field_by_name("EquivalentChainLength"),
                        col(RETENTION_TIME)
                            .struct_()
                            .field_by_name("Absolute")
                            .struct_()
                            .field_by_name("Mean"),
                    ],
                    sort_options,
                )
                .over([col("Mode")])]),
    }
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
        // self.clone().saturated_or_null(expr).backward_fill(None)
        ternary_expr(self.is_saturated(), expr, lit(NULL))
            .fill_null_with_strategy(FillNullStrategy::Backward(None))
    }

    fn forward(self, expr: Expr) -> Expr {
        // self.clone().saturated_or_null(expr).forward_fill(None)
        ternary_expr(self.is_saturated(), expr, lit(NULL))
            .fill_null_with_strategy(FillNullStrategy::Forward(None))
    }
}

pub(crate) mod display;
// pub(crate) mod plot;

// /// Replace unsaturated with null
// pub fn saturated_or_null(self, expr: Expr) -> Expr {
//     ternary_expr(self.is_saturated(), expr, lit(NULL))
// }

// /// Replace saturated with null
// pub fn unsaturated_or_null(self, expr: Expr) -> Expr {
//     ternary_expr(self.is_saturated().not(), expr, lit(NULL))
// }
