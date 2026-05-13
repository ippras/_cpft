use crate::{
    app::{
        computers::{distance::process::OUTPUT_SCHEMA as INPUT_SCHEMA, matches_schema},
        states::distance::Settings,
    },
    r#const::{
        ABSOLUTE, ARRAY, CHAIN_LENGTH, DEAD_TIME, DISTANCE, EQUIVALENT_CHAIN_LENGTH, FILTER,
        FRACTIONAL_CHAIN_LENGTH, MASS, MEAN, MODE, RELATIVE, RETENTION_FACTOR, RETENTION_TIME,
        SELECTIVITY_FACTOR, STANDARD_DEVIATION, TEMPERATURE,
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
        lazy_frame = lazy_frame.select([dtype_cols(&[DataType::Float64, DataType::String])
            .as_selector()
            .as_expr()]);
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
            ddof: settings.ddof,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Export distance value
type Value = HashedDataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        // .unnest(
        //     cols([MODE, FATTY_ACID, RETENTION_TIME, EQUIVALENT_CHAIN_LENGTH]),
        //     Some(PlSmallStr::from_static(".")),
        // )
        .with_columns([
            col(formatcp!(r#"^{FATTY_ACID}\..+$"#))
                .fatty_acid()
                .display(),
            cols([DEAD_TIME, formatcp!(r#"^{MASS}\..+$"#)])
                .as_expr()
                .precision(key.precision, key.significant),
        ]);
    let names = [
        formatcp!("{RETENTION_TIME}.{DISTANCE}"),
        formatcp!("{EQUIVALENT_CHAIN_LENGTH}.{DISTANCE}"),
        SELECTIVITY_FACTOR,
    ];
    lazy_frame = lazy_frame.with_columns(names.map(|name| {
        let array = Array::builder()
            .expr(col(name))
            .ddof(key.ddof)
            .precision(key.precision)
            .significant(key.significant)
            .build();
        as_struct(vec![
            array
                .clone()
                .struct_()
                .field_by_name(ARRAY)
                .arr()
                .to_struct(Some(PlanCallback::new(move |index| {
                    Ok(format!("{ARRAY}[{index}]"))
                })))
                .struct_()
                .field_by_name("*"),
            array.clone().struct_().field_by_name(MEAN),
            array.clone().struct_().field_by_name(STANDARD_DEVIATION),
        ])
        .alias(name)
    }));
    lazy_frame = lazy_frame.unnest(cols(names), Some(PlSmallStr::from_static(".")));
    Ok(lazy_frame)
}
