use crate::{
    app::{
        MAX_TEMPERATURE,
        computers::matches_schema,
        states::source::{Filter, Settings, Sort, settings::sort::SortKind},
    },
    r#const::{
        ABSOLUTE, ADJUSTED, BACKWARD, CHAIN_LENGTH, CHANGED, DEAD_TIME, EQUIVALENT_CARBON_NUMBER,
        EQUIVALENT_CHAIN_LENGTH, FILTER, FORWARD, FRACTIONAL_CHAIN_LENGTH, MASS, MODE,
        ONSET_TEMPERATURE, RANK, RELATIVE, RETENTION_FACTOR, RETENTION_TIME, SELECTIVITY_FACTOR,
        STANDARD, TEMPERATURE, TEMPERATURE_STEP,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::sync::LazyLock;
use widgets::settings::order::OrderKind;

fn array_of_nulls<const N: usize>() -> Expr {
    lit(Scalar::new_array(Series::new_null(PlSmallStr::EMPTY, N), N))
}

/// Input schema
pub(crate) static INPUT_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
        // Индекс порядка элюирования
        Field::new(PlSmallStr::from_static(INDEX), DataType::UInt32),
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
        field!(FATTY_ACID),
        Field::new(
            PlSmallStr::from_static(RETENTION_TIME),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(PlSmallStr::from_static(DEAD_TIME), DataType::Float64),
    ]))
});

/// Output schema
pub(crate) static OUTPUT_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
        Field::new(PlSmallStr::from_static(INDEX), DataType::UInt32),
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
        field!(FATTY_ACID),
        Field::new(PlSmallStr::from_static(DEAD_TIME), DataType::Float64),
        Field::new(
            PlSmallStr::from_static(RETENTION_TIME),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(ABSOLUTE),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(RELATIVE),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(ADJUSTED),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(RETENTION_FACTOR),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(SELECTIVITY_FACTOR),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(FORWARD),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(BACKWARD),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(CHAIN_LENGTH),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(EQUIVALENT_CHAIN_LENGTH),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(FRACTIONAL_CHAIN_LENGTH),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(EQUIVALENT_CARBON_NUMBER),
                    DataType::UInt8,
                ),
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(TEMPERATURE),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(MASS),
            DataType::Struct(vec![
                Field::new(PlSmallStr::from_static("RCO"), DataType::Float64),
                Field::new(PlSmallStr::from_static("RCOO"), DataType::Float64),
                Field::new(PlSmallStr::from_static("RCOOH"), DataType::Float64),
                Field::new(PlSmallStr::from_static("RCOOCH3"), DataType::Float64),
            ]),
        ),
        Field::new(PlSmallStr::from_static(FILTER), DataType::Boolean),
        // _
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{FORWARD}{ONSET_TEMPERATURE}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{BACKWARD}{ONSET_TEMPERATURE}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{FORWARD}{TEMPERATURE_STEP}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{BACKWARD}{TEMPERATURE_STEP}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{ONSET_TEMPERATURE}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}{TEMPERATURE_STEP}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static(formatcp!("_{CHANGED}")),
            DataType::Boolean,
        ),
        Field::new(
            PlSmallStr::from_static("_"),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(formatcp!("_{STANDARD}{RETENTION_TIME}")),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(formatcp!("_{FORWARD}{RETENTION_TIME}")),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(formatcp!("_{BACKWARD}{RETENTION_TIME}")),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(
                    PlSmallStr::from_static(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}")),
                    DataType::Struct(vec![
                        Field::new(
                            PlSmallStr::from_static(formatcp!("{FORWARD}{CARBON}")),
                            DataType::Array(Box::new(DataType::UInt8), 0),
                        ),
                        Field::new(
                            PlSmallStr::from_static(formatcp!("{BACKWARD}{CARBON}")),
                            DataType::Array(Box::new(DataType::UInt8), 0),
                        ),
                        Field::new(
                            PlSmallStr::from_static(formatcp!("{FORWARD}{RETENTION_TIME}")),
                            DataType::Array(Box::new(DataType::Float64), 0),
                        ),
                        Field::new(
                            PlSmallStr::from_static(formatcp!("{BACKWARD}{RETENTION_TIME}")),
                            DataType::Array(Box::new(DataType::Float64), 0),
                        ),
                    ]),
                ),
                Field::new(
                    PlSmallStr::from_static(formatcp!("_{FRACTIONAL_CHAIN_LENGTH}")),
                    DataType::Struct(vec![Field::new(
                        PlSmallStr::from_static(CARBON),
                        DataType::UInt8,
                    )]),
                ),
            ]),
        ),
    ]))
});

/// Source computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        // Filter
        lazy_frame = filter(lazy_frame, key)?;
        // Interpolate
        // Sort
        lazy_frame = sort(lazy_frame, key)?;
        // Select
        lazy_frame = lazy_frame.select([
            col(INDEX),
            col(MODE),
            col(FATTY_ACID),
            col(DEAD_TIME),
            col(RETENTION_TIME),
            col(RETENTION_FACTOR),
            col(SELECTIVITY_FACTOR),
            col(CHAIN_LENGTH),
            col(TEMPERATURE),
            col(MASS),
            col(FILTER),
            // _
            // Changed
            col(formatcp!("_{CHANGED}{FORWARD}{ONSET_TEMPERATURE}")),
            col(formatcp!("_{CHANGED}{BACKWARD}{ONSET_TEMPERATURE}")),
            col(formatcp!("_{CHANGED}{FORWARD}{TEMPERATURE_STEP}")),
            col(formatcp!("_{CHANGED}{BACKWARD}{TEMPERATURE_STEP}")),
            col(formatcp!("_{CHANGED}{ONSET_TEMPERATURE}")),
            col(formatcp!("_{CHANGED}{TEMPERATURE_STEP}")),
            col(formatcp!("_{CHANGED}")),
            // Changed
            as_struct(vec![
                col(formatcp!("_{STANDARD}{RETENTION_TIME}")),
                col(formatcp!("_{FORWARD}{RETENTION_TIME}")),
                col(formatcp!("_{BACKWARD}{RETENTION_TIME}")),
                col(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}")),
                col(formatcp!("_{FRACTIONAL_CHAIN_LENGTH}")),
            ])
            .alias("_"),
        ]);
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
    pub(crate) order: OrderKind,
    pub(crate) relative: &'a Option<FattyAcid>,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.mean_and_standard_deviation.ddof,
            filter: &settings.filter,
            logarithmic: settings.logarithmic,
            order: settings.order.kind,
            relative: &settings.relative,
            sort: settings.sort,
        }
    }
}

/// Source value
type Value = HashedDataFrame;

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Retention time
    lazy_frame = lazy_frame
        .with_column(_standard_retention_time(key)?)
        .with_columns([
            col(RETENTION_TIME).alias(ABSOLUTE),
            relative_retention_time().alias(RELATIVE),
            adjusted_retention_time().alias(ADJUSTED),
        ]);
    // Factors
    lazy_frame = lazy_frame
        .with_columns([_forward_retention_time(), _backward_retention_time()])
        .with_columns([
            retention_factor().alias(RETENTION_FACTOR),
            selectivity_factor()?.alias(SELECTIVITY_FACTOR),
        ]);
    // Chain length
    lazy_frame = lazy_frame
        .with_column(_equivalent_chain_length()?)
        .with_column(equivalent_chain_length(key)?.alias(EQUIVALENT_CHAIN_LENGTH))
        .with_column(_fractional_chain_length()?)
        .with_columns([
            fractional_chain_length().alias(FRACTIONAL_CHAIN_LENGTH),
            equivalent_carbon_number().alias(EQUIVALENT_CARBON_NUMBER),
        ]);
    lazy_frame = lazy_frame.with_columns([temperature()?.alias(TEMPERATURE)]);
    lazy_frame = _changed(lazy_frame);
    lazy_frame = lazy_frame.with_columns([
        // Retention time
        as_struct(vec![col(ABSOLUTE), col(RELATIVE), col(ADJUSTED)]).alias(RETENTION_TIME),
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
    ]);
    Ok(lazy_frame)
}

fn _changed(mut lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame = lazy_frame.with_columns([col(INDEX)
        .rank(RankOptions::default(), None)
        .over([col(MODE)])
        .alias(RANK)]);
    // Ищем изменения вдоль оси t0
    // Сортируем данные так, чтобы для каждого ЖК и каждого dt температуры t0 шли по порядку
    lazy_frame = lazy_frame
        .sort_by_exprs(
            vec![
                col(FATTY_ACID),
                col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
                col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
            ],
            SortMultipleOptions::default(),
        )
        .with_columns([
            col(RANK)
                .neq(col(RANK).shift(lit(-1)))
                .over([
                    col(FATTY_ACID),
                    col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
                ])
                .fill_null(lit(false))
                .alias(formatcp!("_{CHANGED}{FORWARD}{ONSET_TEMPERATURE}")),
            col(RANK)
                .neq(col(RANK).shift(lit(1)))
                .over([
                    col(FATTY_ACID),
                    col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
                ])
                .fill_null(lit(false))
                .alias(formatcp!("_{CHANGED}{BACKWARD}{ONSET_TEMPERATURE}")),
        ]);
    // Ищем изменения вдоль оси dt
    // Пересортировываем данные, чтобы теперь шаги dt шли по порядку для каждого t0
    lazy_frame = lazy_frame
        .sort_by_exprs(
            vec![
                col(FATTY_ACID),
                col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
                col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
            ],
            SortMultipleOptions::default(),
        )
        .with_columns([
            col(RANK)
                .neq(col(RANK).shift(lit(-1)))
                .over([
                    col(FATTY_ACID),
                    col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
                ])
                .fill_null(lit(false))
                .alias(formatcp!("_{CHANGED}{FORWARD}{TEMPERATURE_STEP}")),
            col(RANK)
                .neq(col(RANK).shift(lit(1)))
                .over([
                    col(FATTY_ACID),
                    col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
                ])
                .fill_null(lit(false))
                .alias(formatcp!("_{CHANGED}{BACKWARD}{TEMPERATURE_STEP}")),
        ]);
    lazy_frame = lazy_frame
        .with_columns([
            col(formatcp!("_{CHANGED}{FORWARD}{ONSET_TEMPERATURE}"))
                .or(col(formatcp!("_{CHANGED}{BACKWARD}{ONSET_TEMPERATURE}")))
                .alias(formatcp!("_{CHANGED}{ONSET_TEMPERATURE}")),
            col(formatcp!("_{CHANGED}{FORWARD}{TEMPERATURE_STEP}"))
                .or(col(formatcp!("_{CHANGED}{BACKWARD}{TEMPERATURE_STEP}")))
                .alias(formatcp!("_{CHANGED}{TEMPERATURE_STEP}")),
        ])
        .with_column(
            col(formatcp!("_{CHANGED}{ONSET_TEMPERATURE}"))
                .or(formatcp!("_{CHANGED}{TEMPERATURE_STEP}"))
                .alias(formatcp!("_{CHANGED}")),
        );
    lazy_frame.sort([INDEX], SortMultipleOptions::default())
}

// Standard retention time
fn _standard_retention_time(key: Key) -> PolarsResult<Expr> {
    // Время удерживания стандарта по отношению к которому будет расчитано
    // относительное время удерживания.
    Ok(match &key.relative {
        Some(fatty_acid) => {
            let fatty_acid = FattyAcidExpr::try_from(fatty_acid)?;
            eval_arr(col(RETENTION_TIME), |element| {
                Ok(element
                    .filter(col(FATTY_ACID).fatty_acid().equal(fatty_acid.clone()))
                    .first())
            })?
            .over([MODE])
        }
        None => lit(Scalar::null(DataType::Array(
            Box::new(DataType::Float64),
            3,
        ))),
    }
    .alias(formatcp!("_{STANDARD}{RETENTION_TIME}")))
}

fn _equivalent_chain_length() -> PolarsResult<Expr> {
    let forward_carbon = eval_arr(col(RETENTION_TIME), |element| {
        Ok(forward(
            col(FATTY_ACID).fatty_acid().carbon(),
            Some(
                col(FATTY_ACID)
                    .fatty_acid()
                    .is_saturated()
                    .and(element.is_not_null()),
            ),
        )
        .over([MODE]))
    })?
    .alias(formatcp!("{FORWARD}{CARBON}"));
    let backward_carbon = eval_arr(col(RETENTION_TIME), |element| {
        Ok(backward(
            col(FATTY_ACID).fatty_acid().carbon(),
            Some(
                col(FATTY_ACID)
                    .fatty_acid()
                    .is_saturated()
                    .and(element.is_not_null()),
            ),
        )
        .over([MODE]))
    })?
    .alias(formatcp!("{BACKWARD}{CARBON}"));
    let forward_retention_time = eval_arr(col(RETENTION_TIME), |element| {
        Ok(forward(element, Some(col(FATTY_ACID).fatty_acid().is_saturated())).over([MODE]))
    })?
    .alias(formatcp!("{FORWARD}{RETENTION_TIME}"));
    let backward_retention_time = eval_arr(col(RETENTION_TIME), |element| {
        Ok(backward(element, Some(col(FATTY_ACID).fatty_acid().is_saturated())).over([MODE]))
    })?
    .alias(formatcp!("{BACKWARD}{RETENTION_TIME}"));
    Ok(as_struct(vec![
        forward_carbon,
        backward_carbon,
        forward_retention_time,
        backward_retention_time,
    ])
    .alias(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}")))
}

fn _fractional_chain_length() -> PolarsResult<Expr> {
    let carbon = col(FATTY_ACID).fatty_acid().carbon();
    Ok(as_struct(vec![carbon]).alias(formatcp!("_{FRACTIONAL_CHAIN_LENGTH}")))
}

// Forward retention time
fn _forward_retention_time() -> Expr {
    // Сдвигаем "вниз", чтобы получить предыдущую строку (вперед, по направлению к началу серии)
    col(ABSOLUTE)
        .shift(lit(1))
        .fill_null(array_of_nulls::<3>())
        .alias(formatcp!("_{FORWARD}{RETENTION_TIME}"))
}

// Backward retention time
fn _backward_retention_time() -> Expr {
    // Сдвигаем "вверх", чтобы получить следующую строку (назад, по направлению к концу серии)
    col(ABSOLUTE)
        .shift(lit(-1))
        .fill_null(array_of_nulls::<3>())
        .alias(formatcp!("_{BACKWARD}{RETENTION_TIME}"))
}

// Adjusted retention time
fn adjusted_retention_time() -> Expr {
    col(RETENTION_TIME) - col(DEAD_TIME)
}

// Relative retention time
fn relative_retention_time() -> Expr {
    col(RETENTION_TIME) / col(formatcp!("_{STANDARD}{RETENTION_TIME}"))
}

// Retention factor
fn retention_factor() -> Expr {
    col(ADJUSTED) / col(DEAD_TIME)
}

// Selectivity factor
fn selectivity_factor() -> PolarsResult<Expr> {
    Ok(as_struct(vec![
        (col(ABSOLUTE) / col(formatcp!("_{FORWARD}{RETENTION_TIME}"))).alias(FORWARD),
        (col(formatcp!("_{BACKWARD}{RETENTION_TIME}")) / col(ABSOLUTE)).alias(BACKWARD),
    ])
    .over([MODE]))
}

/// Temperature
fn temperature() -> PolarsResult<Expr> {
    eval_arr(col(RETENTION_TIME), |element| {
        Ok((col(MODE).struct_().field_by_name(ONSET_TEMPERATURE)
            + element * col(MODE).struct_().field_by_name(TEMPERATURE_STEP))
        .clip_max(lit(MAX_TEMPERATURE)))
    })
}

// FCL
fn fractional_chain_length() -> Expr {
    col(EQUIVALENT_CHAIN_LENGTH)
        - col(formatcp!("_{FRACTIONAL_CHAIN_LENGTH}"))
            .struct_()
            .field_by_name(CARBON)
}

// ECL
fn equivalent_chain_length(key: Key) -> PolarsResult<Expr> {
    Ok(when(col(FATTY_ACID).fatty_acid().is_saturated())
        .then(concat_arr(vec![col(FATTY_ACID).fatty_acid().carbon(); 3])?)
        .otherwise({
            let retention_time = col(RETENTION_TIME);
            let forward_carbon = col(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                .struct_()
                .field_by_name(formatcp!("{FORWARD}{CARBON}"));
            let backward_carbon = col(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                .struct_()
                .field_by_name(formatcp!("{BACKWARD}{CARBON}"));
            let forward_retention_time = col(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                .struct_()
                .field_by_name(formatcp!("{FORWARD}{RETENTION_TIME}"));
            let backward_retention_time = col(formatcp!("_{EQUIVALENT_CHAIN_LENGTH}"))
                .struct_()
                .field_by_name(formatcp!("{BACKWARD}{RETENTION_TIME}"));
            forward_carbon.clone()
                + (backward_carbon - forward_carbon)
                    * (retention_time - forward_retention_time.clone())
                    / (backward_retention_time - forward_retention_time)
        }))
}

// ECN
fn equivalent_carbon_number() -> Expr {
    col(FATTY_ACID).fatty_acid().equivalent_carbon_number()
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

fn sort(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let sort_options =
        SortMultipleOptions::new().with_order_descending(key.order == OrderKind::Descending);
    Ok(match key.sort.kind {
        SortKind::FattyAcid => lazy_frame.sort_by_exprs(
            [
                col(MODE),
                col(FATTY_ACID).fatty_acid().carbon(),
                col(FATTY_ACID).fatty_acid().unsaturation(),
                col(FATTY_ACID).fatty_acid().indices(),
            ],
            sort_options,
        ),
        SortKind::RetentionTime => lazy_frame.sort([MODE, INDEX], sort_options),
    })
}

// Предыдущее (вперед, по направлению к началу серии)
fn forward(mut expr: Expr, mask: Option<Expr>) -> Expr {
    if let Some(mask) = mask {
        expr = expr.nullify(mask);
    }
    expr.fill_null_with_strategy(FillNullStrategy::Forward(None))
}

// Следующее (назад, по направлению к концу серии)
fn backward(mut expr: Expr, mask: Option<Expr>) -> Expr {
    if let Some(mask) = mask {
        expr = expr.nullify(mask);
    }
    expr.fill_null_with_strategy(FillNullStrategy::Backward(None))
        .shift(lit(-1))
}
