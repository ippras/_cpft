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
        // Schema {
        //     fields: {
        //         "Mode": Struct({'OnsetTemperature': Float64, 'TemperatureStep': Float64}),
        //         "FattyAcid": Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}),
        //         "RetentionTime": Struct({'Absolute': Array(Float64, 3), 'Relative': Array(Float64, 3), 'Delta': Array(Float64, 3)}),
        //         "DeadTime": Float64,
        //         "Temperature": Array(Float64, 3),
        //         "ChainLength": Struct({'EquivalentChainLength': Array(Float64, 3), 'FractionalChainLength': Array(Float64, 3), 'EquivalentCarbonNumber': UInt8}),
        //         "Mass": Struct({'RCO': Float64, 'RCOO': Float64, 'RCOOH': Float64, 'RCOOCH3': Float64}),
        //         "Derivative": Struct({'Slope': Array(Float64, 3), 'Angle': Array(Float64, 3)}),
        //         "Filter": Boolean,
        //     },
        //     metadata: (),
        // }
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame.filter(col(FILTER));
        // Join
        lazy_frame = join(lazy_frame, key)?;
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

fn join(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // lazy_frame = lazy_frame
    //     .clone()
    //     .select([
    //         col(MODE).alias("LeftKey"),
    //         as_struct(vec![
    //             col(FATTY_ACID),
    //             col(RETENTION_TIME)
    //                 .struct_()
    //                 .field_by_name(ABSOLUTE)
    //                 .struct_()
    //                 .field_by_name(MEAN)
    //                 .name()
    //                 .keep(),
    //             col(CHAIN_LENGTH)
    //                 .struct_()
    //                 .field_by_name(EQUIVALENT_CHAIN_LENGTH),
    //         ])
    //         .alias(FROM),
    //         col(MODE),
    //         col(DEAD_TIME),
    //     ])
    //     .with_row_index("LeftIndex", None)
    //     .join_builder()
    //     .with(
    //         lazy_frame
    //             .select([
    //                 col(MODE).alias("RightKey"),
    //                 as_struct(vec![
    //                     col(FATTY_ACID),
    //                     col(RETENTION_TIME)
    //                         .struct_()
    //                         .field_by_name(ABSOLUTE)
    //                         .struct_()
    //                         .field_by_name(MEAN)
    //                         .name()
    //                         .keep(),
    //                     col(CHAIN_LENGTH)
    //                         .struct_()
    //                         .field_by_name(EQUIVALENT_CHAIN_LENGTH),
    //                 ])
    //                 .alias(TO),
    //             ])
    //             .with_row_index("RightIndex", None),
    //     )
    //     .join_where(vec![
    //         // Same modes
    //         col("LeftKey").eq(col("RightKey")),
    //         // Fatty asids not equals combination
    //         col("LeftIndex").lt(col("RightIndex")),
    //     ]);
    let retention_time = col(RETENTION_TIME).struct_().field_by_name(ABSOLUTE);
    let fatty_acid = as_struct(vec![
        col(FATTY_ACID),
        retention_time.clone().name().keep(),
        col(CHAIN_LENGTH)
            .struct_()
            .field_by_name(EQUIVALENT_CHAIN_LENGTH),
    ]);
    // ВАЖНО: Сортируем данные, чтобы гарантировать последовательность по времени удерживания
    lazy_frame = lazy_frame.sort_by_exprs(
        vec![col(MODE), retention_time.arr().mean()],
        SortMultipleOptions::default(),
    );
    lazy_frame = lazy_frame.select([
        col(MODE),
        col(DEAD_TIME),
        // Колонка FROM: Текущая строка
        fatty_acid.clone().alias(FROM),
        // Колонка TO: Следующая строка (сдвиг на -1)
        fatty_acid
            .shift(lit(-1)) // Сдвигаем "вверх", чтобы получить следующую строку в текущей позиции
            .over([MODE])? // Группируем по MODE, чтобы конец одной группы не соединился с началом другой
            .alias(TO),
    ]);
    // Убираем последние строки в каждой группе, у которых нет пары (TO is null)
    lazy_frame = lazy_frame.filter(col(TO).is_not_null());
    println!("!!!!!!!!!1: {}", lazy_frame.clone().collect().unwrap());
    // Restructure
    lazy_frame = lazy_frame
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
                .over([MODE])?
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
                .over([MODE])?
                .alias(DELTA),
            ])
            .alias(EQUIVALENT_CHAIN_LENGTH),
            ((col(FROM).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                / (col(TO).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                    .over([MODE])?)
            .alias(ALPHA),
        ])
        .with_column(
            (col(RETENTION_TIME)
                .struct_()
                .field_by_name(DELTA)
                .arr()
                .eval(element().pow(2), false)
                + col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(DELTA)
                    .arr()
                    .eval(element().pow(2), false))
            .arr()
            .eval(element().sqrt(), false)
            .alias(EUCLIDEAN),
        );
    Ok(lazy_frame)
}
