use crate::{
    app::states::distance::{Aggregation, Filter, Order, Priority, Settings, Sort},
    r#const::*,
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
    pub(crate) filter: &'a Filter,
    pub(crate) order: Order,
    pub(crate) priority: Priority,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            filter: &settings.filter,
            order: settings.order,
            priority: settings.priority,
            sort: settings.sort,
        }
    }
}

/// Distance filtered value
type Value = HashedDataFrame;

// fn filter(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
//     let mut expr = None;
//     if !key.filter.onset_temperatures.is_empty() {
//         for &onset_temperature in &key.filter.onset_temperatures {
//             expr = Some(
//                 expr.unwrap_or(lit(true)).and(
//                     col(MODE)
//                         .struct_()
//                         .field_by_name(ONSET_TEMPERATURE)
//                         .neq(onset_temperature),
//                 ),
//             );
//         }
//     }
//     if !key.filter.temperature_steps.is_empty() {
//         for &temperature_step in &key.filter.temperature_steps {
//             expr = Some(
//                 expr.unwrap_or(lit(true)).and(
//                     col(MODE)
//                         .struct_()
//                         .field_by_name(TEMPERATURE_STEP)
//                         .neq(temperature_step),
//                 ),
//             );
//         }
//     }
//     if !key.filter.fatty_acids.is_empty() {
//         for fatty_acid in &key.filter.fatty_acids {
//             expr = Some(
//                 expr.unwrap_or(lit(true))
//                     .and(
//                         col(FATTY_ACID)
//                             .struct_()
//                             .field_by_name(FROM)
//                             .fatty_acid()
//                             .equal(FattyAcidExpr::try_from(fatty_acid)?)
//                             .not(),
//                     )
//                     .and(
//                         col(FATTY_ACID)
//                             .struct_()
//                             .field_by_name(TO)
//                             .fatty_acid()
//                             .equal(FattyAcidExpr::try_from(fatty_acid)?)
//                             .not(),
//                     ),
//             );
//         }
//     }
//     if let Some(predicate) = expr {
//         lazy_frame = lazy_frame.filter(predicate);
//     }
//     Ok(lazy_frame)
// }

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
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if key.order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    lazy_frame.sort_by_exprs(
        match key.sort {
            Sort::Key => vec![
                col(MODE),
                col(FATTY_ACID).struct_().field_by_name(FROM),
                col(FATTY_ACID).struct_().field_by_name(TO),
            ],
            Sort::Value => vec![col(ALPHA).aggregate(key.priority)],
        },
        sort_options,
    )
}

/// Extension methods for [`Expr`]
trait ExprExt {
    fn aggregate(self, priority: Priority) -> Expr;
}

impl ExprExt for Expr {
    fn aggregate(self, priority: Priority) -> Expr {
        match priority.aggregation {
            Aggregation::Maximum => self.abs().max(),
            Aggregation::Mean => self.abs().mean(),
            Aggregation::Median => self.abs().median(),
            Aggregation::Minimum => self.abs().min(),
        }
        .over([MODE])
    }
}
