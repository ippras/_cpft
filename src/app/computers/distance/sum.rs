use crate::{
    app::states::distance::{Aggregation, Distance, Filter, Order, Priority, Settings, Sort},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprExt as _;

/// Distance sum computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance sum computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        println!("lazy_frame SUM 0: {}", lazy_frame.clone().collect()?);
        // Group
        lazy_frame = group(lazy_frame);
        println!("lazy_frame SUM 1: {}", lazy_frame.clone().collect()?);
        // Sort
        lazy_frame = sort(lazy_frame, key);
        // Format
        lazy_frame = format(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("compute distance filtered")
    }
}

/// Distance sum key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) filter: &'a Filter,
    pub(crate) order: Order,
    pub(crate) precision: usize,
    pub(crate) priority: Priority,
    pub(crate) significant: bool,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            filter: &settings.filter,
            order: settings.order,
            precision: settings.precision,
            priority: settings.priority,
            significant: false,
            sort: settings.sort,
        }
    }
}

/// Distance sum value
type Value = HashedDataFrame;

// fn aggregate(name: &str, aggregation: Aggregation) -> Expr {
//     match aggregation {
//         Aggregation::Maximum => self.abs().max(),
//         Aggregation::Median => self.abs().median(),
//         Aggregation::Minimum => self.abs().min(),
//     }
//     .over([col(MODE)])
// }

/// Group
fn group(lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame.group_by([col(MODE)]).agg([
        col(ALPHA).abs().max().name().suffix(".Max"),
        col(ALPHA).abs().mean().name().suffix(".Mean"),
        col(ALPHA).abs().median().name().suffix(".Median"),
        col(ALPHA).abs().min().name().suffix(".Min"),
        col(EQUIVALENT_CHAIN_LENGTH)
            .struct_()
            .field_by_name(DELTA)
            .abs()
            .max()
            .name()
            .keep()
            .name()
            .suffix(".Max"),
        col(EQUIVALENT_CHAIN_LENGTH)
            .struct_()
            .field_by_name(DELTA)
            .abs()
            .mean()
            .name()
            .keep()
            .name()
            .suffix(".Mean"),
        col(EQUIVALENT_CHAIN_LENGTH)
            .struct_()
            .field_by_name(DELTA)
            .abs()
            .median()
            .name()
            .keep()
            .name()
            .suffix(".Median"),
        col(EQUIVALENT_CHAIN_LENGTH)
            .struct_()
            .field_by_name(DELTA)
            .abs()
            .min()
            .name()
            .keep()
            .name()
            .suffix(".Min"),
        col(EUCLIDEAN).abs().max().name().suffix(".Max"),
        col(EUCLIDEAN).abs().mean().name().suffix(".Mean"),
        col(EUCLIDEAN).abs().median().name().suffix(".Median"),
        col(EUCLIDEAN).abs().min().name().suffix(".Min"),
    ])
}

/// Sort
fn sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
    if key.order == Order::Descending {
        sort_options = sort_options.with_order_descending(true);
    };
    lazy_frame.sort_by_exprs(
        match key.sort {
            Sort::Key => vec![col(MODE)],
            Sort::Value => vec![col(key.priority.id())],
        },
        sort_options,
    )
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([all()
        .exclude_cols([MODE])
        .as_expr()
        .precision(key.precision, key.significant)])
}

// /// Extension methods for [`Expr`]
// trait ExprExt {
//     fn aggregate(self, aggregation: Aggregation) -> Expr;
// }
// impl ExprExt for Expr {
//     fn aggregate(self, aggregation: Aggregation) -> Expr {
//         match aggregation {
//             Aggregation::Maximum => self.abs().max(),
//             Aggregation::Median => self.abs().median(),
//             Aggregation::Minimum => self.abs().min(),
//         }
//         .over([col(MODE)])
//     }
// }
