use crate::{
    app::states::source::Settings,
    r#const::*,
    utils::{hash::HashedDataFrame, polars::Array},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::{array::ArrayNameSpace, *};
use polars_ext::expr::eval_arr;
use scirs2::stats::{RegressionResults, StatsAnalyzer, regression::linear_regression};
use scirs2_core::{
    ArrayView2,
    ndarray::{Array1, Array2},
};
use std::iter::zip;

const VALUES_CAPACITY: usize = 3;

/// Regression computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Regression computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame
            .filter(
                col(FATTY_ACID)
                    .fatty_acid()
                    .is_saturated()
                    .and(col(MODE).struct_().field_by_name(ONSET_TEMPERATURE).eq(60))
                    .and(col(MODE).struct_().field_by_name(TEMPERATURE_STEP).eq(1)),
            )
            .filter(col(FATTY_ACID).fatty_acid().equal(C17.clone()).not())
            .select([
                col(MODE),
                col(FATTY_ACID)
                    .fatty_acid()
                    .carbon()
                    .alias(CARBON)
                    .cast(DataType::Float64),
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(ABSOLUTE)
                    .alias(RETENTION_TIME),
            ]);
        println!(
            "lazy_frame Regression Filter: {}",
            lazy_frame.clone().collect()?
        );
        lazy_frame = lazy_frame.select([eval_arr(col(RETENTION_TIME), |expr| {
            as_struct(vec![col(CARBON), expr]).apply(regression::<3>, |_schema, _field| {
                Ok(Field::new(
                    PlSmallStr::EMPTY,
                    DataType::Array(Box::new(DataType::Float64), VALUES_CAPACITY),
                ))
            })
        })?]);
        // lazy_frame = lazy_frame.select([as_struct(vec![col(CARBON), col(RETENTION_TIME)])
        //     .apply(regression, |_schema, _field| {
        //         Ok(Field::new(
        //             PlSmallStr::EMPTY,
        //             DataType::Array(Box::new(DataType::Float64), VALUES_CAPACITY),
        //         ))
        //     })
        //     .alias("Predicted")]);
        println!(
            "lazy_frame Regression Regression: {}",
            lazy_frame.clone().collect()?
        );
        // let regression_results = regression(lazy_frame.clone())?;

        // Group
        lazy_frame = group(lazy_frame)?;
        println!(
            "lazy_frame Regression Group: {}",
            lazy_frame.clone().collect()?
        );
        // Format
        lazy_frame = format(lazy_frame, key);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

// ┌──────────────────────────────────────┐
// │ Carbon                               │
// │ ---                                  │
// │ array[struct[3], 1]                  │
// ╞══════════════════════════════════════╡
// │ [{93.582533,93.441499,93.432202}]    │
// │ [{100.324122,100.172561,100.161635}] │
// │ [{106.832201,106.668968,106.65635}]  │
// │ [{140.97695,140.719689,140.695526}]  │
// └──────────────────────────────────────┘

// 17:0[93.931, 93.771, 93.759]
// 18:0[100.671, 100.469, 100.461
// 19:0[—, 106.87, 106.858]

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).expect("Compute source correlation")
    }
}

/// Regression key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Regression value
type Value = HashedDataFrame;

// fn regression(column: Column) -> PolarsResult<Column> {
//     let fields = column.struct_()?.fields_as_series();
//     let carbons = fields[0].f64()?.into_no_null_iter();
//     let capacity = carbons.len();
//     let mut v: Vec<f64> = Vec::with_capacity(capacity * 2);
//     for value in carbons {
//         v.push(1.0); // x^0
//         v.push(value); // x^1
//     }
//     let x = Array2::from_shape_vec((capacity, 2), v)
//         .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//     let mut array_builder = ListPrimitiveChunkedBuilder::<Float64Type>::new(
//         PlSmallStr::EMPTY,
//         capacity,
//         3,
//         DataType::Float64,
//     );
//     for retention_time_opt_series in fields[1].array()? {
//         let Some(retention_time_series) = retention_time_opt_series else {
//             array_builder.append_null();
//             continue;
//         };
//         let retention_times = retention_time_series.f64()?.into_no_null_iter().collect();
//         println!("retention_times: {retention_times:?}");
//         let y = Array1::from_vec(retention_times);
//         let results = linear_regression(&x.view(), &y.view(), None)
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Summary: {}", results.summary());
//         // Предсказание
//         let x = Array2::from_shape_vec((2, 2), vec![1.0, 17.0, 1.0, 25.0])
//             .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//         let y = results
//             .predict(&x.view())
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Predicted = {}: {}", x[[0, 1]], y[0]);
//         println!("Predicted = {}: {}", x[[1, 1]], y[1]);
//         let series = Series::new(PlSmallStr::from_static("Predicted"), y.to_vec());
//         array_builder.append_series(&series)?;
//     }
//     Ok(array_builder.finish().into_column())
// }

// fn regression(column: Column) -> PolarsResult<Column> {
//     let carbon_series = column.struct_()?.field_by_name(CARBON)?;
//     let carbons = carbon_series.f64()?.into_no_null_iter();
//     let capacity = carbons.len();
//     let mut v: Vec<f64> = Vec::with_capacity(capacity * 2);
//     for value in carbons {
//         v.push(1.0); // x^0
//         v.push(value); // x^1
//     }
//     let x = Array2::from_shape_vec((capacity, 2), v)
//         .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//     let mut array_builder = ListPrimitiveChunkedBuilder::<Float64Type>::new(
//         PlSmallStr::EMPTY,
//         2,
//         VALUES_CAPACITY,
//         DataType::Float64,
//     );
//     let retention_time_array_series = column.struct_()?.field_by_name(RETENTION_TIME)?;
//     for index in 0..VALUES_CAPACITY {
//         let indices = Int64Chunked::new(PlSmallStr::EMPTY, &[index as i64]);
//         let retention_time_series = retention_time_array_series
//             .array()?
//             .array_get(&indices, false)?;
//         // println!("retention_times: {index} {retention_time_series}");
//         let retention_times = retention_time_series.f64()?.into_no_null_iter().collect();
//         println!("retention_times: {retention_times:?}");
//         let y = Array1::from_vec(retention_times);
//         let results = linear_regression(&x.view(), &y.view(), None)
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Summary: {}", results.summary());
//         // Предсказание
//         let x = Array2::from_shape_vec((2, 2), vec![1.0, 17.0, 1.0, 25.0])
//             .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//         let y = results
//             .predict(&x.view())
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Predicted = {}: {}", x[[0, 1]], y[0]);
//         println!("Predicted = {}: {}", x[[1, 1]], y[1]);
//         let series = Series::new(PlSmallStr::from_static("Predicted"), y.to_vec());
//         array_builder.append_series(&series)?;
//     }
//     Ok(array_builder.finish().into_column())
// }

// fn regression(column: Column) -> PolarsResult<Column> {
//     let carbon_series = column.struct_()?.field_by_name(CARBON)?;
//     let carbons = carbon_series.f64()?.into_no_null_iter();
//     let capacity = carbons.len();
//     let mut v: Vec<f64> = Vec::with_capacity(capacity * 2);
//     for value in carbons {
//         v.push(1.0); // x^0
//         v.push(value); // x^1
//     }
//     let x = Array2::from_shape_vec((capacity, 2), v)
//         .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//     let mut array_builder = ListPrimitiveChunkedBuilder::<Float64Type>::new(
//         PlSmallStr::EMPTY,
//         2,
//         VALUES_CAPACITY,
//         DataType::Float64,
//     );
//     let builders = [
//         ListPrimitiveChunkedBuilder::<Float64Type>::new(
//             PlSmallStr::EMPTY,
//             2,
//             VALUES_CAPACITY,
//             DataType::Float64,
//         ),
//         ListPrimitiveChunkedBuilder::<Float64Type>::new(
//             PlSmallStr::EMPTY,
//             2,
//             VALUES_CAPACITY,
//             DataType::Float64,
//         ),
//         ListPrimitiveChunkedBuilder::<Float64Type>::new(
//             PlSmallStr::EMPTY,
//             2,
//             VALUES_CAPACITY,
//             DataType::Float64,
//         ),
//     ];
//     let retention_time_array_series = column.struct_()?.field_by_name(RETENTION_TIME)?;
//     for retention_time_series in retention_time_array_series.array()?.into_no_null_iter() {
//         // builders[0].
//     }
//     for index in 0..VALUES_CAPACITY {
//         let indices = Int64Chunked::new(PlSmallStr::EMPTY, &[index as i64]);
//         let retention_time_series = retention_time_array_series
//             .array()?
//             .array_get(&indices, false)?;
//         // println!("retention_times: {index} {retention_time_series}");
//         let retention_times = retention_time_series.f64()?.into_no_null_iter().collect();
//         println!("retention_times: {retention_times:?}");
//         let y = Array1::from_vec(retention_times);
//         let results = linear_regression(&x.view(), &y.view(), None)
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Summary: {}", results.summary());
//         // Предсказание
//         let x = Array2::from_shape_vec((2, 2), vec![1.0, 17.0, 1.0, 25.0])
//             .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
//         let y = results
//             .predict(&x.view())
//             .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
//         println!("Predicted = {}: {}", x[[0, 1]], y[0]);
//         println!("Predicted = {}: {}", x[[1, 1]], y[1]);
//         let series = Series::new(PlSmallStr::from_static("Predicted"), y.to_vec());
//         array_builder.append_series(&series)?;
//     }
//     Ok(array_builder.finish().into_column())
// }

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

fn regression<const N: usize>(column: Column) -> PolarsResult<Column> {
    let length = 4;

    let fields = column.struct_()?.fields_as_series();
    // let carbons = fields[0].f64()?.into_no_null_iter();
    // let capacity = carbons.len();
    // let mut v: Vec<f64> = Vec::with_capacity(capacity * 2);
    // for value in carbons {
    //     v.push(1.0); // x^0
    //     v.push(value); // x^1
    // }
    // let x = Array2::from_shape_vec((capacity, 2), v)
    //     .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
    let fields = fields[1..]
        .iter()
        .map(|field| {
            let mut carbons = Vec::new();
            let mut retention_times = Vec::new();
            for (carbon, retention_time) in zip(fields[0].f64()?, field.f64()?)
                .filter_map(|(carbon, retention_time)| Some((carbon?, retention_time?)))
            {
                for n in 0..N {
                    carbons.push(carbon.powi(n as _)); // x^n
                }
                retention_times.push(retention_time); // y
            }
            let results = {
                let x = Array2::from_shape_vec((carbons.len() / N, N), carbons)
                    .map_err(|error| polars_err!(ShapeMismatch: error.to_string()))?;
                let y = Array1::from_vec(retention_times);
                linear_regression(&x.view(), &y.view(), Some(0.9999))
                    .map_err(|error| polars_err!(ComputeError: error.to_string()))?
            };
            println!("\nSummary: {}", results.summary());
            // Предсказание
            let x = x::<N>(vec![17.0, 18.0, 19.0, 25.0].into_iter())?;
            let y = results
                .predict(&x.view())
                .map_err(|error| polars_err!(ComputeError: error.to_string()))?;
            println!("Predicted = {}: {}", x[[0, 1]], y[0]);
            println!("Predicted = {}: {}", x[[1, 1]], y[1]);
            Ok(Series::new(field.name().clone(), y.to_vec()))
        })
        .collect::<PolarsResult<Vec<_>>>()?;
    Ok(StructChunked::from_series(PlSmallStr::EMPTY, length, fields.iter())?.into_column())
}

fn regression1(lazy_frame: LazyFrame) -> PolarsResult<RegressionResults<f64>> {
    let data_frame = lazy_frame.collect()?;
    let carbons = data_frame[CARBON].cast(&DataType::Float64)?;
    let carbons = carbons.f64()?.into_no_null_iter();
    let len = carbons.len();
    let mut v: Vec<f64> = Vec::with_capacity(len * 2);
    for value in carbons {
        v.push(1.0); // x^0
        v.push(value); // x^1
    }
    let x = Array2::from_shape_vec((len, 2), v).unwrap();
    let retention_times = data_frame[RETENTION_TIME]
        .f64()?
        .into_no_null_iter()
        .collect();
    let y = Array1::from_vec(retention_times);
    let results = linear_regression(&x.view(), &y.view(), None).unwrap();
    Ok(results)
}

fn regression2(lazy_frame: LazyFrame) -> PolarsResult<RegressionResults<f64>> {
    let data_frame = lazy_frame.collect()?;
    let carbons = data_frame[CARBON].cast(&DataType::Float64)?;
    let carbons = carbons.f64()?.into_no_null_iter();
    let len = carbons.len();
    let mut v: Vec<f64> = Vec::with_capacity(len * 2);
    for value in carbons {
        v.push(1.0); // x^0
        v.push(value); // x^1
        v.push(value.powi(2)); // x^2
    }
    let x = Array2::from_shape_vec((len, 3), v).unwrap();
    let y = Array1::from_iter(data_frame[RETENTION_TIME].f64()?.into_no_null_iter());
    let results = linear_regression(&x.view(), &y.view(), None).unwrap();
    Ok(results)
}

/// Group
fn group(lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame
        .group_by_stable([MODE, FATTY_ACID])
        .agg([
            eval_arr(col(RETENTION_TIME), |expr| pearson_corr(col(CARBON), expr))?
                .alias(CORRELATION),
        ]))
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([Array::builder()
        .expr(col(CORRELATION))
        .ddof(key.ddof)
        .precision(key.precision)
        .significant(key.significant)
        .build()
        .alias(CORRELATION)])
}
