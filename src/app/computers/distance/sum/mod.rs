use crate::{
    app::states::distance::{Aggregation, Distance, Filter, Order, Priority, Settings, Sort},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use polars_ext::prelude::*;

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
        lazy_frame = group(lazy_frame)?;
        println!("lazy_frame SUM 1: {}", lazy_frame.clone().collect()?);
        // Sort
        lazy_frame = sort(lazy_frame, key);
        println!("lazy_frame SUM 2: {}", lazy_frame.clone().collect()?);
        // Format
        lazy_frame = format(lazy_frame, key);
        println!("lazy_frame SUM 3: {}", lazy_frame.clone().collect()?);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("Compute distance sum")
    }
}

/// Distance sum key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
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
            ddof: settings.ddof,
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

/// Group
fn group(lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame.group_by([col(MODE)]).agg([
        as_struct(vec![
            eval_arr(col(ALPHA), |element| Ok(element.abs().max()))?.alias(MAXIMUM),
            eval_arr(col(ALPHA), |element| Ok(element.abs().mean()))?.alias(MEAN),
            eval_arr(col(ALPHA), |element| Ok(element.abs().median()))?.alias(MEDIAN),
            eval_arr(col(ALPHA), |element| Ok(element.abs().min()))?.alias(MINIMUM),
        ])
        .name()
        .keep(),
        as_struct(vec![
            eval_arr(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(DELTA),
                |element| Ok(element.abs().max()),
            )?
            .alias(MAXIMUM),
            eval_arr(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(DELTA),
                |element| Ok(element.abs().mean()),
            )?
            .alias(MEAN),
            eval_arr(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(DELTA),
                |element| Ok(element.abs().median()),
            )?
            .alias(MEDIAN),
            eval_arr(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(DELTA),
                |element| Ok(element.abs().min()),
            )?
            .alias(MINIMUM),
        ])
        .name()
        .keep(),
        as_struct(vec![
            eval_arr(col(EUCLIDEAN), |element| Ok(element.abs().max()))?.alias(MAXIMUM),
            eval_arr(col(EUCLIDEAN), |element| Ok(element.abs().mean()))?.alias(MEAN),
            eval_arr(col(EUCLIDEAN), |element| Ok(element.abs().median()))?.alias(MEDIAN),
            eval_arr(col(EUCLIDEAN), |element| Ok(element.abs().min()))?.alias(MINIMUM),
        ])
        .name()
        .keep(),
    ]))
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
            Sort::Value => vec![
                col(key.priority.distance.id())
                    .struct_()
                    .field_by_name(key.priority.aggregation.id()),
            ],
        },
        sort_options,
    )
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        as_struct(vec![
            mean_and_standard_deviation_and_array(col(ALPHA).struct_().field_by_name(MAXIMUM), key)
                .alias(MAXIMUM),
            mean_and_standard_deviation_and_array(col(ALPHA).struct_().field_by_name(MEAN), key)
                .alias(MEAN),
            mean_and_standard_deviation_and_array(col(ALPHA).struct_().field_by_name(MEDIAN), key)
                .alias(MEDIAN),
            mean_and_standard_deviation_and_array(col(ALPHA).struct_().field_by_name(MINIMUM), key)
                .alias(MINIMUM),
        ])
        .name()
        .keep(),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(MAXIMUM),
                key,
            )
            .alias(MAXIMUM),
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(MEAN),
                key,
            )
            .alias(MEAN),
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH).struct_().field_by_name(MEDIAN),
                key,
            )
            .alias(MEDIAN),
            mean_and_standard_deviation_and_array(
                col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(MINIMUM),
                key,
            )
            .alias(MINIMUM),
        ])
        .name()
        .keep(),
        as_struct(vec![
            mean_and_standard_deviation_and_array(
                col(EUCLIDEAN).struct_().field_by_name(MAXIMUM),
                key,
            )
            .alias(MAXIMUM),
            mean_and_standard_deviation_and_array(
                col(EUCLIDEAN).struct_().field_by_name(MEAN),
                key,
            )
            .alias(MEAN),
            mean_and_standard_deviation_and_array(
                col(EUCLIDEAN).struct_().field_by_name(MEDIAN),
                key,
            )
            .alias(MEDIAN),
            mean_and_standard_deviation_and_array(
                col(EUCLIDEAN).struct_().field_by_name(MINIMUM),
                key,
            )
            .alias(MINIMUM),
        ])
        .name()
        .keep(),
    ])
}

fn mean_and_standard_deviation_and_array(expr: Expr, key: Key) -> Expr {
    Array::builder()
        .expr(expr)
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
}
