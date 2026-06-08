use crate::{
    app::{
        computers::{distance::process::OUTPUT_SCHEMA as INPUT_SCHEMA, matches_schema},
        states::distance::Settings,
    },
    r#const::{
        ABSOLUTE, ARRAY, CHAIN_LENGTH, DEAD_TIME, DISTANCE, EQUIVALENT_CHAIN_LENGTH, FILTER,
        FRACTIONAL_CHAIN_LENGTH, FROM, MASS, MEAN, MODE, ONSET_TEMPERATURE, RELATIVE,
        RETENTION_FACTOR, RETENTION_TIME, SELECTIVITY_FACTOR, STANDARD_DEVIATION, TEMPERATURE,
        TEMPERATURE_STEP, TO,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use tracing::instrument;

/// Export distance computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Export distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        // lazy_frame = lazy_frame.filter(col(FILTER));
        // Format
        lazy_frame = format(lazy_frame, key)?;
        // Select
        lazy_frame = lazy_frame.select([
            col(formatcp!("{MODE}.{ONSET_TEMPERATURE}")),
            col(formatcp!("{MODE}.{TEMPERATURE_STEP}")),
            col(formatcp!("{FATTY_ACID}.{FROM}")),
            col(formatcp!("{FATTY_ACID}.{TO}")),
            col(DEAD_TIME),
            col(formatcp!("{RETENTION_TIME}.{ARRAY}[0]")),
            col(formatcp!("{RETENTION_TIME}.{ARRAY}[1]")),
            col(formatcp!("{RETENTION_TIME}.{ARRAY}[2]")),
            col(formatcp!("{RETENTION_TIME}.{MEAN}")),
            col(formatcp!("{RETENTION_TIME}.{STANDARD_DEVIATION}")),
            col(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{ARRAY}[0]")),
            col(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{ARRAY}[1]")),
            col(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{ARRAY}[2]")),
            col(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{MEAN}")),
            col(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{STANDARD_DEVIATION}")),
            col(formatcp!("{SELECTIVITY_FACTOR}.{ARRAY}[0]")),
            col(formatcp!("{SELECTIVITY_FACTOR}.{ARRAY}[1]")),
            col(formatcp!("{SELECTIVITY_FACTOR}.{ARRAY}[2]")),
            col(formatcp!("{SELECTIVITY_FACTOR}.{MEAN}")),
            col(formatcp!("{SELECTIVITY_FACTOR}.{STANDARD_DEVIATION}")),
        ]);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Export distance key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.mean_and_standard_deviation.ddof,
            precision: settings.precision_and_significant.precision,
            significant: settings.precision_and_significant.significant,
        }
    }
}

/// Export distance value
type Value = HashedDataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .with_columns([
            col(MODE)
                .struct_()
                .field_by_name("*")
                .name()
                .prefix(formatcp!("{MODE}.")),
            col(FATTY_ACID)
                .struct_()
                .field_by_name("*")
                .fatty_acid()
                .delta()
                .name()
                .prefix(formatcp!("{FATTY_ACID}.")),
            col(DEAD_TIME).precision(key.precision, key.significant),
            format_array(
                col(RETENTION_TIME)
                    .struct_()
                    .field_by_name(DISTANCE)
                    .name()
                    .keep(),
                key,
                true,
            ),
            format_array(
                col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(DISTANCE)
                    .name()
                    .keep(),
                key,
                true,
            ),
            format_array(col(SELECTIVITY_FACTOR), key, true),
        ])
        .unnest(
            cols([RETENTION_TIME, EQUIVALENT_CHAIN_LENGTH, SELECTIVITY_FACTOR]),
            Some(PlSmallStr::from_static(".")),
        );

    // let exprs = [
    //     col(RETENTION_TIME).struct_().field_by_name(DISTANCE),
    //     col(EQUIVALENT_CHAIN_LENGTH)
    //         .struct_()
    //         .field_by_name(DISTANCE),
    //     col(SELECTIVITY_FACTOR),
    // ];
    // lazy_frame = lazy_frame.with_columns(exprs.map(|expr| {
    //     let array = Array::builder()
    //         .expr(expr)
    //         .ddof(key.ddof)
    //         .precision(key.precision)
    //         .significant(key.significant)
    //         .build();
    //     as_struct(vec![
    //         array
    //             .clone()
    //             .struct_()
    //             .field_by_name(ARRAY)
    //             .arr()
    //             .to_struct(Some(PlanCallback::new(move |index| {
    //                 Ok(format!("{ARRAY}[{index}]"))
    //             })))
    //             .struct_()
    //             .field_by_name("*"),
    //         array.clone().struct_().field_by_name(MEAN),
    //         array.clone().struct_().field_by_name(STANDARD_DEVIATION),
    //     ])
    //     .name()
    //     .keep()
    // }));
    // lazy_frame = lazy_frame.unnest(
    //     cols([RETENTION_TIME, EQUIVALENT_CHAIN_LENGTH, SELECTIVITY_FACTOR]),
    //     Some(PlSmallStr::from_static(".")),
    // );
    Ok(lazy_frame)
}

fn format_array(expr: Expr, key: Key, flatten: bool) -> Expr {
    let mut array = expr.clone().arr().eval(
        element()
            // .percent(key.percent)
            .precision(key.precision, key.significant),
        false,
    );
    if flatten {
        array = array
            .arr()
            .to_struct(Some(PlanCallback::new(move |index| {
                Ok(format!("{ARRAY}[{index}]"))
            })))
            .struct_()
            .field_by_name("*");
    } else {
        array = array.alias(ARRAY);
    }
    as_struct(vec![
        array,
        expr.clone()
            .arr()
            .mean()
            // .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias(MEAN),
        expr.clone()
            .arr()
            .std(key.ddof)
            // .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias(STANDARD_DEVIATION),
    ])
    .name()
    .keep()
    // Array::builder()
    //     .expr(expr)
    //     .ddof(key.ddof)
    //     .precision(key.precision)
    //     .significant(key.significant)
    //     .build()
}
