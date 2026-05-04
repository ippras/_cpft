use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::Settings,
    },
    r#const::{CHAIN_LENGTH, DERIVATIVE, MASS, MODE, RETENTION_TIME},
    utils::{hash::HashedDataFrame, polars::Array},
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use tracing::instrument;

// let mut data = data_frame
//     .lazy()
//     .select([
//         col(MODE).struct_().field_by_name("*"),
//         col(FATTY_ACID),
//         col(RETENTION_TIME)
//             .struct_()
//             .field_by_name(ABSOLUTE)
//             .struct_()
//             .field_by_name(MEAN)
//             .name()
//             .keep(),
//         col(DEAD_TIME),
//     ])
//     .with_row_index("Index", None)
//     .collect()?;

/// Flat export calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Flat export calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = format(lazy_frame, key)?;
        lazy_frame = filter_and_sort(lazy_frame, key);
        lazy_frame = lazy_frame.select([dtype_cols(&[DataType::Float64, DataType::String])
            .as_selector()
            .as_expr()]);
        // println!("EXPORT 3: {}", lazy_frame.clone().collect()?);
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Flat export calculation key
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

/// Flat export calculation value
type Value = HashedDataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame.unnest(
        cols([MODE, RETENTION_TIME, CHAIN_LENGTH, MASS, DERIVATIVE]),
        Some(PlSmallStr::from_static(".")),
    );
    println!("EXPORT7: {}", lazy_frame.clone().collect()?);
    lazy_frame = lazy_frame.with_columns([
        dtype_col(&DataType::Float64).as_selector().as_expr(),
        dtype_col(&DataType::Array(Box::new(DataType::Float64), 3))
            .as_selector()
            .as_expr(),
        col(formatcp!(r#"{RETENTION_TIME}\..+"#)),
        col(FATTY_ACID).fatty_acid().display(),
    ]);
    Ok(lazy_frame)
}

// fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
//     // Stereospecific numbers
//     lazy_frame = lazy_frame
//         .with_columns(
//             FlatArray::builder()
//                 .expr(col(STEREOSPECIFIC_NUMBERS123))
//                 .ddof(key.ddof)
//                 .percent(key.percent)
//                 .precision(key.precision)
//                 .significant(key.significant)
//                 .build()?,
//         )
//         .with_columns(
//             FlatArray::builder()
//                 .expr(col(STEREOSPECIFIC_NUMBERS13))
//                 .ddof(key.ddof)
//                 .percent(key.percent)
//                 .precision(key.precision)
//                 .significant(key.significant)
//                 .build()?,
//         )
//         .with_columns(
//             FlatArray::builder()
//                 .expr(col(STEREOSPECIFIC_NUMBERS2))
//                 .ddof(key.ddof)
//                 .percent(key.percent)
//                 .precision(key.precision)
//                 .significant(key.significant)
//                 .build()?,
//         );
//     // Enrichment factor
//     let mut enrichment_factor = FattyAcidExpr::enrichment_factor(
//         col(STEREOSPECIFIC_NUMBERS2),
//         col(STEREOSPECIFIC_NUMBERS123),
//     )
//     .alias(formatcp!("{ENRICHMENT}{FACTOR}"));
//     if key.percent {
//         enrichment_factor = enrichment_factor / lit(3);
//     }
//     lazy_frame = lazy_frame.with_columns(
//         FlatArray::builder()
//             .expr(enrichment_factor)
//             .ddof(key.ddof)
//             .percent(key.percent)
//             .precision(key.precision)
//             .significant(key.significant)
//             .build()?,
//     );
//     // Selectivity factor
//     let mut selectivity_factor = concat_arr(vec![
//         col(FATTY_ACID).fatty_acid().selectivity_factor(
//             col(STEREOSPECIFIC_NUMBERS2)
//                 .arr()
//                 .to_struct(None)
//                 .struct_()
//                 .field_by_name("*"),
//             col(STEREOSPECIFIC_NUMBERS123)
//                 .arr()
//                 .to_struct(None)
//                 .struct_()
//                 .field_by_name("*"),
//         ),
//     ])?
//     .alias(formatcp!("{SELECTIVITY}{FACTOR}"));
//     if key.percent {
//         selectivity_factor = selectivity_factor / lit(3);
//     }
//     lazy_frame = lazy_frame.with_columns(
//         FlatArray::builder()
//             .expr(selectivity_factor)
//             .ddof(key.ddof)
//             .percent(key.percent)
//             .precision(key.precision)
//             .significant(key.significant)
//             .build()?,
//     );
//     lazy_frame = lazy_frame.with_columns([col(FATTY_ACID).fatty_acid().display()]);
//     Ok(lazy_frame)
// }

// Filter and sort threshold (major, minor)
fn filter_and_sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    // if key.threshold.filter {
    //     lazy_frame.filter(col(THRESHOLD))
    // } else if key.threshold.sort {
    //     lazy_frame.sort_by_exprs(
    //         [col(THRESHOLD)],
    //         SortMultipleOptions::default()
    //             .with_maintain_order(true)
    //             .with_order_reversed(),
    //     )
    // } else {
    //     lazy_frame
    // }
    lazy_frame
}
