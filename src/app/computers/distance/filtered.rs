use crate::{
    app::states::distance::{Aggregation, Filter, Order, Settings, SortBy},
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;

/// Distance filtered computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance filtered computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = filter(lazy_frame, key)?;
        // Sort
        lazy_frame = sort(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("compute distance filtered")
    }
}

/// Distance filtered key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) aggregation: Aggregation,
    pub(crate) filter: &'a Filter,
    pub(crate) order: Order,
    pub(crate) sort: SortBy,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            aggregation: settings.sort.aggregation,
            filter: &settings.filter,
            order: settings.sort.order,
            sort: settings.sort.by,
        }
    }
}

/// Distance filtered value
type Value = HashedDataFrame;

fn filter(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut expr = None;
    if !key.filter.onset_temperatures.is_empty() {
        for &onset_temperature in &key.filter.onset_temperatures {
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
    if !key.filter.temperature_steps.is_empty() {
        for &temperature_step in &key.filter.temperature_steps {
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
    if !key.filter.fatty_acids.is_empty() {
        for fatty_acid in &key.filter.fatty_acids {
            expr = Some(
                expr.unwrap_or(lit(true))
                    .and(
                        col("FattyAcid")
                            .struct_()
                            .field_by_name("From")
                            .fatty_acid()
                            .equal(FattyAcidExpr::try_from(fatty_acid)?)
                            .not(),
                    )
                    .and(
                        col("FattyAcid")
                            .struct_()
                            .field_by_name("To")
                            .fatty_acid()
                            .equal(FattyAcidExpr::try_from(fatty_acid)?)
                            .not(),
                    ),
            );
        }
    }
    if let Some(predicate) = expr {
        lazy_frame = lazy_frame.filter(predicate);
    }
    Ok(lazy_frame)
}

fn sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if key.order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    lazy_frame.sort_by_exprs(
        match key.sort {
            SortBy::Key => vec![
                col("Mode"),
                col("FattyAcid").struct_().field_by_name("From"),
                col("FattyAcid").struct_().field_by_name("To"),
            ],
            SortBy::Value => vec![col("Alpha").aggregate(key.aggregation)],
        },
        sort_options,
    )
}

/// Extension methods for [`Expr`]
trait ExprExt {
    fn aggregate(self, aggregation: Aggregation) -> Expr;
}

impl ExprExt for Expr {
    fn aggregate(self, aggregation: Aggregation) -> Expr {
        match aggregation {
            Aggregation::Maximum => self.abs().max(),
            Aggregation::Median => self.abs().median(),
            Aggregation::Minimum => self.abs().min(),
        }
        .over([col("Mode")])
    }
}
