use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::{Settings, settings::Regression},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::{expr::ExprExt, prelude::*};
use polyfit::{
    ChebyshevFit, LogarithmicFit, PhysicistsHermiteFit, basis_select, plot,
    score::ScoringMethod::Aic, statistics::DegreeBound,
};
use std::{
    collections::{BTreeMap, HashMap},
    iter::zip,
    mem::MaybeUninit,
    sync::LazyLock,
};
use tracing::debug;

pub(crate) const X1: &str = "x[1]";
pub(crate) const X2: &str = "x[2]";
pub(crate) const Y: &str = "y";
pub(crate) const Z: &str = "z";

const I: usize = 3;

const FALSE_REGRESSION: LazyLock<Scalar> =
    LazyLock::new(|| Scalar::new_array(Series::new(PlSmallStr::EMPTY, &[false; I]), I));
const TRUE_REGRESSION: LazyLock<Scalar> =
    LazyLock::new(|| Scalar::new_array(Series::new(PlSmallStr::EMPTY, &[true; I]), I));

/// Regression computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Regression computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = lazy_frame.select([all().exclude_cols(["^_.*$"]).as_expr()]);
        println!("regression 0: {}", lazy_frame.clone().collect().unwrap());

        // // Filter
        // let filtered = &key
        //     .frame
        //     .data_frame
        //     .filter(key.frame.data_frame[FILTER].bool()?)?[FATTY_ACID];
        // let n_unique = filtered.n_unique()?;
        // if n_unique != 1 {
        //     return Ok((
        //         HashedDataFrame::EMPTY,
        //         Default::default(),
        //         Default::default(),
        //     ));
        //     // return Err(polars_err!(ComputeError: "n_unique: {n_unique}"));
        // }
        // lazy_frame = lazy_frame.filter(col(FILTER));
        // println!("regression 1: {}", lazy_frame.clone().collect().unwrap());

        // Rename
        lazy_frame = lazy_frame.select([
            col(MODE)
                .struct_()
                .field_by_name(TEMPERATURE_STEP)
                .alias(X1),
            col(MODE)
                .struct_()
                .field_by_name(ONSET_TEMPERATURE)
                .alias(X2),
            col(FATTY_ACID),
            col(CHAIN_LENGTH)
                .struct_()
                .field_by_name(EQUIVALENT_CHAIN_LENGTH)
                .alias(Y),
        ]);
        println!("regression 1: {}", lazy_frame.clone().collect().unwrap());
        // lazy_frame = lazy_frame.with_column(eval_arr(col(Y), |element| {
        //     Ok(as_struct(vec![element, col(X2)]))
        // })?);
        // lazy_frame = lazy_frame.with_column(as_struct(vec![col(X2), col(Y).explode()]).implode());
        println!("regression 2: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame
            .group_by_stable([FATTY_ACID, X1])
            .agg([col(X2), col(Y)]);
        println!("regression 3: {}", lazy_frame.clone().collect().unwrap());
        let mut lazy_frame = polyfit(lazy_frame)?;
        println!("regression 4: {}", lazy_frame.clone().collect().unwrap());
        // Regression
        let (mut lazy_frame, parameters, metrics) = regression::<3>(lazy_frame)?;
        // Format
        lazy_frame = format(lazy_frame, key);

        // Rename
        lazy_frame = lazy_frame.select([
            as_struct(vec![
                col(X1).alias(ONSET_TEMPERATURE),
                col(X2).alias(TEMPERATURE_STEP),
            ])
            .alias(MODE),
            col(FATTY_ACID),
            col(Y).alias(EQUIVALENT_CHAIN_LENGTH),
            col(Z).alias(REGRESSION),
        ]);
        Ok((
            HashedDataFrame::new(lazy_frame.collect()?)?,
            parameters,
            metrics,
        ))
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
            ddof: settings.mean_and_standard_deviation.ddof,
            precision: settings.precision_and_significant.precision,
            regression: &settings.regression,
            significant: settings.precision_and_significant.significant,
        }
    }
}

/// Regression value
type Value = (HashedDataFrame, Parameters, Metrics<f64>);

/// Regression parameters
type Parameters = BTreeMap<String, Vec<f64>>;

/// Regression metrics
#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics<T> {
    pub mean_absolute_error: [T; I],
    pub mean_squared_error: [T; I],
    pub r2: [T; I],
}

impl Metrics<f64> {
    fn uninit() -> Metrics<MaybeUninit<f64>> {
        Metrics {
            mean_absolute_error: [MaybeUninit::uninit(); I],
            mean_squared_error: [MaybeUninit::uninit(); I],
            r2: [MaybeUninit::uninit(); I],
        }
    }
}

impl Metrics<MaybeUninit<f64>> {
    unsafe fn assume_init(self) -> Metrics<f64> {
        Metrics {
            mean_absolute_error: unsafe {
                MaybeUninit::array_assume_init(self.mean_absolute_error)
            },
            mean_squared_error: unsafe { MaybeUninit::array_assume_init(self.mean_squared_error) },
            r2: unsafe { MaybeUninit::array_assume_init(self.r2) },
        }
    }
}

fn polyfit(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    let data_frame = lazy_frame.clone().collect()?;
    let fatty_acid = data_frame[FATTY_ACID].fatty_acid();
    let x1 = data_frame[X1].f64()?;
    let x2 = data_frame[X2].list()?;
    let y = data_frame[Y].list()?;
    let mut coeffs_builder = ListPrimitiveChunkedBuilder::<Float64Type>::new(
        PlSmallStr::EMPTY,
        data_frame.height(),
        data_frame.height() * 5,
        DataType::Float64,
    );
    for row in 0..data_frame.height() {
        let Some(fatty_acid) = fatty_acid.get(row)? else {
            coeffs_builder.append_null();
            continue;
        };
        if fatty_acid.unsaturated.is_empty() {
            continue;
        }
        let fatty_acid = fatty_acid.id();
        let Some(x1) = x1.get(row) else {
            coeffs_builder.append_null();
            continue;
        };
        let Some(x2) = x2.get_as_series(row) else {
            coeffs_builder.append_null();
            continue;
        };
        let Some(y) = y.get_as_series(row) else {
            coeffs_builder.append_null();
            continue;
        };
        let x2 = x2.f64()?;
        let y = y.array()?;
        let mut rows: Vec<(Option<f64>, [Option<f64>; 3])> = Vec::with_capacity(x2.len());
        for (x2, y) in x2.into_iter().zip(y.into_iter()) {
            let mut y_arr = [None, None, None];
            if let Some(y_inner_series) = y {
                // Извлекаем 3 значения из внутреннего массива
                if let Ok(y_f64) = y_inner_series.f64() {
                    let vals: Vec<Option<f64>> = y_f64.into_iter().collect();
                    for i in 0..3 {
                        if i < vals.len() {
                            y_arr[i] = vals[i];
                        }
                    }
                }
            }
            rows.push((x2, y_arr));
        }

        let mut data: Vec<(f64, f64)> = Vec::with_capacity(rows.len() * 3);
        // ПЕРВЫЙ ПРОХОД (индекс 0 массива Y): ПРЯМОЙ порядок (от 1 до 10)
        for (x_opt, y_arr) in rows.iter() {
            if let (Some(x), Some(y_val)) = (x_opt, y_arr[0]) {
                if !x.is_nan() && !y_val.is_nan() {
                    data.push((*x, y_val));
                }
            }
        }

        // ВТОРОЙ ПРОХОД (индекс 1 массива Y): ОБРАТНЫЙ порядок (от 10 до 1)
        for (x_opt, y_arr) in rows.iter().rev() {
            if let (Some(x), Some(y_val)) = (x_opt, y_arr[1]) {
                if !x.is_nan() && !y_val.is_nan() {
                    data.push((*x, y_val));
                }
            }
        }

        // ТРЕТИЙ ПРОХОД (индекс 2 массива Y): ПРЯМОЙ порядок (от 1 до 10)
        for (x_opt, y_arr) in rows.iter() {
            if let (Some(x), Some(y_val)) = (x_opt, y_arr[2]) {
                if !x.is_nan() && !y_val.is_nan() {
                    data.push((*x, y_val));
                }
            }
        }

        let title = format!("Chebyshev basis {{{FATTY_ACID}={fatty_acid};{X1}={x1}}}");
        if data.len() >= 3 {
            // basis_select!(&data, DegreeBound::Relaxed, &Aic);
            let fit = PhysicistsHermiteFit::new_auto(&data, DegreeBound::Custom(3), &Aic).unwrap();
            println!("{title}: {fit}");
            // println!("{title}: {:?}|{}", fit.coefficients(), fit.r_squared(None));
            plot!(fit, {
                title: title.clone(),
                x_label: Some(format!("{ONSET_TEMPERATURE}")),
                y_label: Some(format!("{EQUIVALENT_CHAIN_LENGTH}")),
                // size: (1920, 1024),
            }, prefix = title);
        } else {
            // panic!("{title}")
            coeffs_builder.append_null();
        }

        // let title = format!("Logarithmic basis {{{FATTY_ACID}={fatty_acid};{X1}={x1}}}");
        // if data.len() >= 3 {
        //     basis_select!(&data, DegreeBound::Relaxed, &Aic);
        //     // let fit = LogarithmicFit::new_auto(&data, DegreeBound::Custom(1), &Aic).unwrap();
        //     // println!("{title}: {fit}");
        //     // println!("{title}: {:?}|{}", fit.coefficients(), fit.r_squared(None));
        //     // plot!(fit, {
        //     //     title: title.clone(),
        //     //     x_label: Some(format!("{TEMPERATURE_STEP}")),
        //     //     y_label: Some(format!("{EQUIVALENT_CHAIN_LENGTH}")),
        //     //     size: (1920, 1024),
        //     // }, prefix = title);
        // } else {
        //     // panic!("{title}")
        //     coeffs_builder.append_null();
        // }
    }
    Ok(lazy_frame)
}

fn regression<const N: usize>(
    mut lazy_frame: LazyFrame,
) -> PolarsResult<(LazyFrame, Parameters, Metrics<f64>)> {
    // X (records, двумерная)
    let x = lazy_frame.clone().select(x_exprs::<N>()).collect()?;
    debug!("x: {x}");
    let records = x.to_ndarray::<Float64Type>(IndexOrder::C)?;

    // Y (targets, одномерная)
    let y = lazy_frame
        .clone()
        .select([col(Y)
            .arr()
            .to_struct(Some(PlanCallback::new(move |index| {
                Ok(format!("{Y}[{index}]"))
            })))
            .struct_()
            .field_by_name("*")])
        .collect()?;
    debug!("y: {y}");
    let targets = y.to_ndarray::<Float64Type>(IndexOrder::C)?;

    let mut z = Vec::with_capacity(I);
    let mut metrics = Metrics::uninit();
    let mut parameters = Parameters::new();
    for index in 0..I {
        let dataset = Dataset::new(records.clone(), targets.column(index).to_owned());
        let model = LinearRegression::new()
            .fit(&dataset)
            .map_err(|error| polars_err!(ComputeError: "{error}"))?;
        debug!("Модель успешно обучена!");
        // println!("Смещение (b0): {:.6}", model.intercept());
        for (index, parameter) in model.params().iter().enumerate() {
            parameters
                .entry(x[index].name().to_string())
                .or_default()
                .push(*parameter);
        }
        // Предсказание (интерполяция и экстраполяция)
        let predictions = model.predict(&records);
        debug!("predictions: {predictions}");
        // Считаем стандартные метрики
        metrics.mean_absolute_error[index].write(
            predictions
                .mean_absolute_error(&dataset)
                .map_err(|error| polars_err!(ComputeError: "{error}"))?,
        );
        metrics.mean_squared_error[index].write(
            predictions
                .mean_squared_error(&dataset)
                .map_err(|error| polars_err!(ComputeError: "{error}"))?,
        );
        metrics.r2[index].write(
            predictions
                .r2(&dataset)
                .map_err(|error| polars_err!(ComputeError: "{error}"))?,
        );
        z.push(lit(Series::from_iter(predictions)))
    }
    lazy_frame = lazy_frame.with_column(concat_arr(z)?.alias(Z));
    let metrics = unsafe { metrics.assume_init() };
    Ok((lazy_frame, parameters, metrics))
}

// // 1-я степень
// col(X1),
// col(X2),
// // 2-я степень
// col(X1).pow(2).alias(formatcp!("{X1}^2")),
// col(X2).pow(2).alias(formatcp!("{X2}^2")),
// (col(X1) * col(X2)).alias(formatcp!("{X1}*{X2}")),
// // 3-я степень
// col(X1).pow(3).alias(formatcp!("{X1}^3")),
// col(X2).pow(3).alias(formatcp!("{X2}^3")),
// (col(X1).pow(2) * col(X2)).alias(formatcp!("{X1}^2*{X2}")),
// (col(X1) * col(X2).pow(2)).alias(formatcp!("{X1}*{X2}^2")),
fn x_exprs<const N: usize>() -> Vec<Expr> {
    let mut exprs = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            if 0 < i + j && i + j <= N {
                let x1 = if i == 1 {
                    col(X1)
                } else {
                    col(X1).pow(i as u32)
                };
                let x2 = if j == 1 {
                    col(X2)
                } else {
                    col(X2).pow(j as u32)
                };
                let expr = match (i, j) {
                    (_, 0) => x1,
                    (0, _) => x2,
                    (_, _) => x1 * x2,
                };
                exprs.push(expr.alias(&format!("{X1}^{i}*{X2}^{j}")));
            }
        }
    }
    exprs
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        col(FATTY_ACID).fatty_acid().delta(),
        Array::builder()
            .expr(col(Y))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build(),
        Array::builder()
            .expr(col(Z))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build(),
    ])
}
