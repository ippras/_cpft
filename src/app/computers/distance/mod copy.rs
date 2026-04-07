Все комбинации жирных кислот (старыйй вариант)

use crate::{app::states::distance::Settings, r#const::*, utils::hash::HashedDataFrame};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::FATTY_ACID;
use polars::prelude::*;

/// Distance computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame.filter(col(FILTER));
        // Join
        lazy_frame = join(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        self.try_compute(key).expect("compute distance")
    }
}

/// Distance key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self { frame }
    }
}

/// Distance value
type Value = HashedDataFrame;

fn join(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame = lazy_frame
        .clone()
        .select([
            col(MODE).alias("LeftKey"),
            as_struct(vec![
                col(FATTY_ACID),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .struct_()
                    .field_by_name(MEAN)
                    .name()
                    .keep(),
                col(CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(EQUIVALENT_CHAIN_LENGTH),
            ])
            .alias(FROM),
            col(MODE),
            col(DEAD_TIME),
        ])
        .with_row_index("LeftIndex", None)
        .join_builder()
        .with(
            lazy_frame
                .select([
                    col(MODE).alias("RightKey"),
                    as_struct(vec![
                        col(FATTY_ACID),
                        col(RETENTION_TIME)
                            .struct_()
                            .field_by_name(ABSOLUTE)
                            .struct_()
                            .field_by_name(MEAN)
                            .name()
                            .keep(),
                        col(CHAIN_LENGTH)
                            .struct_()
                            .field_by_name(EQUIVALENT_CHAIN_LENGTH),
                    ])
                    .alias(TO),
                ])
                .with_row_index("RightIndex", None),
        )
        .join_where(vec![
            // Same modes
            col("LeftKey").eq(col("RightKey")),
            // Fatty asids not equals combination
            col("LeftIndex").lt(col("RightIndex")),
        ]);
    // Restructure
    lazy_frame
        .select([
            col(MODE),
            col(DEAD_TIME),
            as_struct(vec![
                col(FROM).struct_().field_by_name(FATTY_ACID).name().keep(),
                col(TO).struct_().field_by_name(FATTY_ACID).name().keep(),
            ])
            .alias(FATTY_ACID),
            as_struct(vec![
                col(FROM)
                    .struct_()
                    .field_by_name(RETENTION_TIME)
                    .name()
                    .keep(),
                col(TO)
                    .struct_()
                    .field_by_name(RETENTION_TIME)
                    .name()
                    .keep(),
                (col(TO).struct_().field_by_name(RETENTION_TIME)
                    - col(FROM).struct_().field_by_name(RETENTION_TIME))
                .over([MODE])
                .alias(DELTA),
            ])
            .alias(RETENTION_TIME),
            as_struct(vec![
                col(FROM)
                    .struct_()
                    .field_by_name(EQUIVALENT_CHAIN_LENGTH)
                    .name()
                    .keep(),
                col(TO)
                    .struct_()
                    .field_by_name(EQUIVALENT_CHAIN_LENGTH)
                    .name()
                    .keep(),
                (col(TO).struct_().field_by_name(EQUIVALENT_CHAIN_LENGTH)
                    - col(FROM).struct_().field_by_name(EQUIVALENT_CHAIN_LENGTH))
                .over([MODE])
                .alias(DELTA),
            ])
            .alias(EQUIVALENT_CHAIN_LENGTH),
            ((col(FROM).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                / (col(TO).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                    .over([MODE]))
            .alias(ALPHA),
        ])
        .with_column(
            (col(RETENTION_TIME).struct_().field_by_name(DELTA).pow(2)
                + col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(DELTA)
                    .pow(2))
            .sqrt()
            .alias(EUCLIDEAN),
        )
}

pub(crate) mod display;
pub(crate) mod filtered;
pub(crate) mod sum;
// pub(crate) mod plot;
