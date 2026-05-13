use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::distance::Settings,
    },
    r#const::{
        ABSOLUTE, CHAIN_LENGTH, DEAD_TIME, DISTANCE, EQUIVALENT_CHAIN_LENGTH, ERROR, FILTER, FROM,
        MODE, ONSET_TEMPERATURE, RETENTION_TIME, SELECTIVITY_FACTOR, TEMPERATURE_STEP, TO, WARNING,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::sync::LazyLock;

/// Output schema
pub(crate) static OUTPUT_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
        Field::new(
            PlSmallStr::from_static(MODE),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(ONSET_TEMPERATURE),
                    DataType::Float64,
                ),
                Field::new(PlSmallStr::from_static(TEMPERATURE_STEP), DataType::Float64),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(FATTY_ACID),
            DataType::Struct(vec![
                Field::new(PlSmallStr::from_static(FROM), data_type!(FATTY_ACID)),
                Field::new(PlSmallStr::from_static(TO), data_type!(FATTY_ACID)),
            ]),
        ),
        Field::new(PlSmallStr::from_static(DEAD_TIME), DataType::Float64),
        Field::new(
            PlSmallStr::from_static(RETENTION_TIME),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(FROM),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(TO),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(DISTANCE),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(EQUIVALENT_CHAIN_LENGTH),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(FROM),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(TO),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(DISTANCE),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(SELECTIVITY_FACTOR),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{WARNING}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{ERROR}")),
            DataType::Boolean,
        ),
    ]))
});

/// Distance computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        // Фильтруем до расчетов дистанций, чтоб считать только необходимые дистанции между жирными кислотами
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
    // // ВАЖНО: Сортируем данные, чтобы гарантировать последовательность по времени удерживания
    // lazy_frame = lazy_frame.sort_by_exprs(
    //     vec![col(MODE), retention_time.arr().mean()],
    //     SortMultipleOptions::default().with_maintain_order(true),
    // );
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
            as_struct(vec![
                col(FROM).struct_().field_by_name(FATTY_ACID).name().keep(),
                col(TO).struct_().field_by_name(FATTY_ACID).name().keep(),
            ])
            .alias(FATTY_ACID),
            col(DEAD_TIME),
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
                .alias(DISTANCE),
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
                .alias(DISTANCE),
            ])
            .alias(EQUIVALENT_CHAIN_LENGTH),
            ((col(FROM).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                / (col(TO).struct_().field_by_name(RETENTION_TIME) - col(DEAD_TIME))
                    .over([MODE])?)
            .alias(SELECTIVITY_FACTOR),
        ])
        .with_columns([
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(DISTANCE)
                .arr()
                .agg(element().lt(0).any(false))
                .alias(formatcp!("_{WARNING}")),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(DISTANCE)
                .arr()
                .agg(element().lt(0).any(false))
                .alias(formatcp!("_{ERROR}")),
        ]);
    Ok(lazy_frame)
}
