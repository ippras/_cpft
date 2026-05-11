use crate::{
    app::states::source::{Regression, Settings},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::{expr::ExprExt, prelude::*};
use scirs2::stats::regression::linear_regression;
use scirs2_core::ndarray::{Array1, Array2};
use std::{iter::zip, sync::LazyLock};

const FALSE_REGRESSION: LazyLock<Scalar> =
    LazyLock::new(|| Scalar::new_array(Series::new(PlSmallStr::EMPTY, &[false; 3]), 3));
const TRUE_REGRESSION: LazyLock<Scalar> =
    LazyLock::new(|| Scalar::new_array(Series::new(PlSmallStr::EMPTY, &[true; 3]), 3));

/// Regression computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Regression computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame.select([
            col(MODE),
            col(FATTY_ACID),
            col(RETENTION_TIME)
                .struct_()
                .field_by_name(ABSOLUTE)
                .alias(RETENTION_TIME),
            col(DEAD_TIME),
        ]);
        println!("filtered: {}", lazy_frame.clone().collect()?);

        let regression = regression::<4>(lazy_frame.clone(), key)?;
        println!("regression: {}", regression.clone().collect()?);

        // std::fs::write(
        //     "REGRESSION.md",
        //     lazy_frame
        //         .clone()
        //         .select([
        //             col(MODE),
        //             col(FATTY_ACID),
        //             concat_arr(vec![
        //                 as_struct(vec![
        //                     col(FATTY_ACID)
        //                         .fatty_acid()
        //                         .carbon()
        //                         .cast(DataType::Float64),
        //                     col(RETENTION_TIME)
        //                         .arr()
        //                         .to_struct(None)
        //                         .struct_()
        //                         .field_by_name("*"),
        //                 ])
        //                 .apply(regression::<4>, |_schema, field| {
        //                     Ok(Field::new(PlSmallStr::EMPTY, field.dtype.clone()))
        //                 })
        //                 .struct_()
        //                 .field_by_name(r#"^field_\d+$"#),
        //             ])?
        //             .over([MODE])
        //             .alias(RETENTION_TIME),
        //         ])
        //         .collect()?
        //         .to_string(),
        // )?;

        // Fill for RETENTION_TIME
        lazy_frame = lazy_frame
            .join(
                regression,
                [col(MODE), col(FATTY_ACID)],
                [col(MODE), col(FATTY_ACID)],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            )
            .with_column(
                col(formatcp!("{RETENTION_TIME}_right"))
                    .fill_null(col(RETENTION_TIME))
                    .alias(RETENTION_TIME),
            )
            .drop(cols([formatcp!("{RETENTION_TIME}_right")]));
        // Fill null for DEAD_TIME
        lazy_frame = lazy_frame.with_columns([
            col(DEAD_TIME)
                .fill_null(col(DEAD_TIME).first_non_null())
                .over([MODE])?,
            col(REGRESSION)
                .fill_null(lit(FALSE_REGRESSION.clone()))
                .over([MODE])?,
        ]);
        lazy_frame = lazy_frame.sort([MODE, RETENTION_TIME], SortMultipleOptions::new());
        println!("lazy_frame: {}", lazy_frame.clone().collect()?);

        // Format
        lazy_frame = format(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("Compute source regression")
    }
}

/// Regression key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) regression: &'a Regression,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            regression: &settings.regression,
            significant: settings.significant,
        }
    }
}

/// Regression value
type Value = HashedDataFrame;

fn regression<const N: usize>(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // of observations (2) must be greater than number of predictors
    // let saturated_fatty_acids = df! {
    //     FATTY_ACID => (key.regression.start..=key.regression.end)
    //         .map(|carbon| {
    //             AnyValue::StructOwned(Box::new((
    //                 vec![
    //                     AnyValue::UInt8(carbon),
    //                     AnyValue::List(Series::new_empty(
    //                         PlSmallStr::from_static(INDICES),
    //                         &data_type!(INDEX),
    //                     )),
    //                 ],
    //                 vec![field!(CARBON), field!(INDICES)],
    //             )))
    //         }).collect::<Vec<_>>(),
    // }?;
    let fatty_acids = key
        .regression
        .fatty_acids
        .iter()
        .filter(|fatty_acid| fatty_acid.unsaturated.is_empty())
        .map(|fatty_acid| {
            AnyValue::StructOwned(Box::new((
                vec![
                    AnyValue::UInt8(fatty_acid.carbon),
                    AnyValue::List(Series::new_empty(
                        PlSmallStr::from_static(INDICES),
                        &data_type!(INDEX),
                    )),
                ],
                vec![field!(CARBON), field!(INDICES)],
            )))
        })
        .collect::<Vec<_>>();
    // Сreates frame with unique modes with predictable saturated fatty acids.
    let cross_joined = lazy_frame
        .clone()
        .select([col(MODE)])
        .unique(None, UniqueKeepStrategy::Any)
        .cross_join(df! { FATTY_ACID => fatty_acids }?.lazy(), None)
        .with_columns([lit(true).alias(REGRESSION)]);
    // Обединяем фрейм с предсказываемыми и фрейм с наблюдаемыми SFA
    lazy_frame = lazy_frame
        .filter(col(FATTY_ACID).fatty_acid().is_saturated())
        .join(
            cross_joined,
            [col(MODE), col(FATTY_ACID)],
            [col(MODE), col(FATTY_ACID)],
            JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .with_columns([col(RETENTION_TIME).nullify(col(REGRESSION).is_null())])
        .sort([MODE, FATTY_ACID], SortMultipleOptions::new());
    println!("sorted: {}", lazy_frame.clone().collect()?);

    // Observations Predictors
    let regression = lazy_frame.select([
        col(MODE),
        col(FATTY_ACID),
        concat_arr(vec![
            as_struct(vec![
                col(FATTY_ACID)
                    .fatty_acid()
                    .carbon()
                    .cast(DataType::Float64),
                col(RETENTION_TIME)
                    .arr()
                    .to_struct(None)
                    .struct_()
                    .field_by_name("*"),
            ])
            .apply(regression_n::<N>, |_schema, field| {
                Ok(Field::new(PlSmallStr::EMPTY, field.dtype.clone()))
            })
            .struct_()
            .field_by_name(r#"^field_\d+$"#),
        ])?
        .over([MODE])?
        .alias(RETENTION_TIME),
        col(RETENTION_TIME)
            .arr()
            .eval(element().is_null(), false)
            .fill_null(lit(TRUE_REGRESSION.clone()))
            .alias(REGRESSION),
    ]);
    Ok(regression)
}

fn x<const N: usize>(values: impl ExactSizeIterator<Item = f64>) -> PolarsResult<Array2<f64>> {
    let mut v: Vec<f64> = Vec::with_capacity(values.len() * N);
    for value in values {
        for n in 0..N {
            v.push(value.powi(n as _)); // x^n
        }
    }
    Array2::from_shape_vec((v.len() / N, N), v)
        .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))
}

fn regression0<const N: usize>(column: Column) -> PolarsResult<Column> {
    let fields = column.struct_()?.fields_as_series();
    let carbon_series = &fields[0];
    let retention_times_series = &fields[1..];
    let mut fields = Vec::with_capacity(fields.len());
    fields.push(carbon_series.clone());
    for retention_time_series in retention_times_series {
        let is_not_null = retention_time_series.is_not_null();
        let is_null = retention_time_series.is_null();

        // Training
        let mut carbons = Vec::new();
        let mut retention_times = Vec::new();
        for (carbon, retention_time) in zip(
            carbon_series
                .filter(&is_not_null)?
                .f64()?
                .into_no_null_iter(),
            retention_time_series
                .filter(&is_not_null)?
                .f64()?
                .into_no_null_iter(),
        ) {
            for n in 0..N {
                carbons.push(carbon.powi(n as _)); // x^n
            }
            retention_times.push(retention_time); // y
        }
        let results = {
            let x = Array2::from_shape_vec((carbons.len() / N, N), carbons)
                .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
            let y = Array1::from_vec(retention_times);
            linear_regression(&x.view(), &y.view(), None)
                .map_err(|error| polars_err!(ComputeError: error.to_string()))?
        };
        // println!("Summary: {}", results.summary());

        // Predict
        let x = x::<N>(carbon_series.filter(&is_null)?.f64()?.into_no_null_iter())?;
        let y = results
            .predict(&x.view())
            .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
        let null_indices =
            IdxCa::from_iter_values(PlSmallStr::EMPTY, 0..(carbon_series.len() as _))
                .filter(&is_null)?
                .into_no_null_iter()
                .collect::<Vec<_>>();
        let values = Float64Chunked::new(PlSmallStr::EMPTY, y.to_vec());
        let retention_time = retention_time_series
            .f64()?
            .clone()
            .scatter(&null_indices, &values)?;
        fields.push(retention_time)
    }
    let r#struct = StructChunked::from_series(column.name().clone(), column.len(), fields.iter())?;
    Ok(r#struct.into_column())
}

fn regression_n<const N: usize>(column: Column) -> PolarsResult<Column> {
    let fields = column.struct_()?.fields_as_series();
    let carbon_series = &fields[0];
    let retention_times_series = &fields[1..];
    let mut fields = Vec::with_capacity(fields.len());
    fields.push(carbon_series.clone());
    for retention_time_series in retention_times_series {
        let is_not_null = retention_time_series.is_not_null();
        let is_null = retention_time_series.is_null();

        // println!(
        //     "carbons (is_not_null): {:?}",
        //     carbon_series
        //         .filter(&is_not_null)?
        //         .f64()?
        //         .into_no_null_iter()
        //         .collect::<Vec<_>>()
        // );
        // println!(
        //     "carbons (is_null): {:?}",
        //     carbon_series
        //         .filter(&is_null)?
        //         .f64()?
        //         .into_no_null_iter()
        //         .collect::<Vec<_>>()
        // );

        // Training
        let mut carbons = Vec::new();
        let mut retention_times = Vec::new();
        for (carbon, retention_time) in zip(
            carbon_series
                .filter(&is_not_null)?
                .f64()?
                .into_no_null_iter(),
            retention_time_series
                .filter(&is_not_null)?
                .f64()?
                .into_no_null_iter(),
        ) {
            for n in 0..N {
                carbons.push(carbon.powi(n as _)); // x^n
            }
            retention_times.push(retention_time); // y
        }
        let results = {
            let x = Array2::from_shape_vec((carbons.len() / N, N), carbons)
                .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
            let y = Array1::from_vec(retention_times);
            linear_regression(&x.view(), &y.view(), None)
                .map_err(|error| polars_err!(ComputeError: error.to_string()))?
        };
        // println!("Summary: {}", results.summary());

        // Predict
        let x = x::<N>(carbon_series.filter(&is_null)?.f64()?.into_no_null_iter())?;
        let y = results
            .predict(&x.view())
            .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
        let null_indices =
            IdxCa::from_iter_values(PlSmallStr::EMPTY, 0..(carbon_series.len() as _))
                .filter(&is_null)?
                .into_no_null_iter()
                .collect::<Vec<_>>();
        let values = Float64Chunked::new(PlSmallStr::EMPTY, y.to_vec());
        let retention_time = retention_time_series
            .f64()?
            .clone()
            .scatter(&null_indices, &values)?;
        fields.push(retention_time)
    }
    let r#struct = StructChunked::from_series(column.name().clone(), column.len(), fields.iter())?;
    Ok(r#struct.into_column())
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        col(MODE),
        col(FATTY_ACID).fatty_acid().display(),
        Array::builder()
            .expr(col(RETENTION_TIME))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build()
            .alias(RETENTION_TIME),
        as_struct(vec![
            col(REGRESSION).arr().agg(element().any(true)).alias(ANY),
            col(REGRESSION).alias(ARRAY),
        ])
        .name()
        .keep(),
    ])
}
