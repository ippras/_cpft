use crate::{
    app::{
        MAX_TEMPERATURE,
        states::source::{Filter, Order, Settings, Sort},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::{ExprExt, eval_arr};

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
    lazy_frame = lazy_frame.with_column(col(RETENTION_TIME).list().to_array(3));
    println!(
        "COMPUTE0 lazy_frame: {}",
        lazy_frame.clone().collect().unwrap()
    );
    lazy_frame = lazy_frame.with_columns([
        col(RETENTION_TIME).alias(ABSOLUTE),
        relative_retention_time(key)?.alias(RELATIVE),
        delta_retention_time()?.alias(DELTA),
        temperature()?.alias(TEMPERATURE),
        fcl(key)?.alias(FRACTIONAL_CHAIN_LENGTH),
        ecl(key)?.alias(EQUIVALENT_CHAIN_LENGTH),
        ecn().alias(EQUIVALENT_CARBON_NUMBER),
    ]);
    lazy_frame = lazy_frame.with_columns([slope(key).alias(SLOPE)]);
    println!(
        "COMPUTE3 lazy_frame: {}",
        lazy_frame.clone().collect().unwrap()
    );
    lazy_frame = lazy_frame.select([
        col(MODE),
        col(FATTY_ACID),
        // Retention time
        as_struct(vec![col(ABSOLUTE), col(RELATIVE), col(DELTA)]).alias(RETENTION_TIME),
        // DeadTime
        col(DEAD_TIME),
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
        as_struct(vec![
            col(SLOPE),
            col(SLOPE)
                .arr()
                .eval(element().arctan().degrees(), false)
                .alias(ANGLE),
        ])
        .alias(DERIVATIVE),
    ]);
    println!(
        "COMPUTE4 lazy_frame: {}",
        lazy_frame.clone().collect().unwrap()
    );
    Ok(lazy_frame)
}

// Relative retention time
fn relative_retention_time(key: Key) -> PolarsResult<Expr> {
    Ok(match &key.relative {
        Some(fatty_acid) => {
            let fatty_acid = FattyAcidExpr::try_from(fatty_acid)?;
            eval_arr(col(RETENTION_TIME), |expr| {
                expr.clone()
                    / expr
                        .filter(col(FATTY_ACID).fatty_acid().equal(fatty_acid.clone()))
                        .first()
            })?
            .over([MODE])
        }
        None => lit(Scalar::null(DataType::Array(
            Box::new(DataType::Float64),
            3,
        ))),
    })
}

// Delta retention time
fn delta_retention_time() -> PolarsResult<Expr> {
    eval_arr(col(RETENTION_TIME), |expr| {
        col(FATTY_ACID).fatty_acid().delta(expr).over([MODE])
    })
}

/// Temperature
fn temperature() -> PolarsResult<Expr> {
    eval_arr(col(RETENTION_TIME), |expr| {
        (col(MODE).struct_().field_by_name(ONSET_TEMPERATURE)
            + expr * col(MODE).struct_().field_by_name(TEMPERATURE_STEP))
        .clip_max(lit(MAX_TEMPERATURE))
    })
}

// FCL
fn fcl(key: Key) -> PolarsResult<Expr> {
    eval_arr(col(RETENTION_TIME), |expr| {
        col(FATTY_ACID)
            .fatty_acid()
            .fractional_chain_length(expr, key.logarithmic)
            .over([MODE])
    })
}

// ECL
fn ecl(key: Key) -> PolarsResult<Expr> {
    eval_arr(col(RETENTION_TIME), |expr| {
        col(FATTY_ACID)
            .fatty_acid()
            .equivalent_chain_length(expr, key.logarithmic)
            .over([MODE])
    })
}

// ECN
fn ecn() -> Expr {
    col(FATTY_ACID).fatty_acid().equivalent_carbon_number()
}

// Slope
fn slope(key: Key) -> Expr {
    if key.relative.is_some() {
        col(FATTY_ACID)
            .fatty_acid()
            .slope(col(EQUIVALENT_CHAIN_LENGTH), col(RELATIVE))
            .over([MODE])
            .arr()
            .eval(element().fill_nan(lit(NULL)), false)
    } else {
        col(FATTY_ACID)
            .fatty_acid()
            .slope(col(EQUIVALENT_CHAIN_LENGTH), col(ABSOLUTE))
            .over([MODE])
            .arr()
            .eval(element().fill_nan(lit(NULL)), false)
    }
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
        .with_maintain_order(true)
        .with_nulls_last(false)
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
                    .arr()
                    .mean()],
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
        self.clone().backward(expr.clone()) - self.forward(expr)
    }

    fn slope(self, dividend: Expr, divisor: Expr) -> Expr {
        // ternary_expr(
        //     self.clone().is_saturated(),
        //     self.clone().delta(dividend) / self.clone().delta(divisor),
        //     self.clone().delta(dividend) / self.delta(divisor),
        // )
        self.clone().delta(dividend) / self.delta(divisor)
    }

    // Следующее по направлению к концу серии
    fn backward(self, expr: Expr) -> Expr {
        expr.shift(lit(-1))
            .nullify(self.is_saturated().shift(lit(-1)))
            .fill_null_with_strategy(FillNullStrategy::Backward(None))
        // ternary_expr(self.is_saturated(), expr, lit(NULL))
        //     .fill_null_with_strategy(FillNullStrategy::Backward(None))
    }

    // Следующее по направлению к началу серии
    fn forward(self, expr: Expr) -> Expr {
        expr.nullify(self.is_saturated())
            .fill_null_with_strategy(FillNullStrategy::Forward(None))
        // ternary_expr(self.is_saturated(), expr, lit(NULL))
        //     .fill_null_with_strategy(FillNullStrategy::Forward(None))
    }
}

pub(crate) mod format;
pub(crate) mod plot;
pub(crate) mod sum;
