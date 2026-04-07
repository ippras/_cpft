use crate::{
    app::{
        computers::{matches_schema, source::process::OUTPUT_SCHEMA as INPUT_SCHEMA},
        states::source::{
            Settings,
            settings::{Export, export::Column},
        },
    },
    r#const::{
        ABSOLUTE, ADJUSTED, ARRAY, BACKWARD, CHAIN_LENGTH, DEAD_TIME, EQUIVALENT_CHAIN_LENGTH,
        FILTER, FORWARD, FRACTIONAL_CHAIN_LENGTH, MASS, MEAN, MODE, RELATIVE,
        RELATIVE_STANDARD_DEVIATION, RETENTION_FACTOR, RETENTION_TIME, SELECTIVITY_FACTOR,
        STANDARD_DEVIATION, TEMPERATURE,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use tracing::instrument;

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
        // Filter
        lazy_frame = lazy_frame.filter(col(FILTER));
        // Format
        lazy_frame = format(lazy_frame, key)?;
        // Select
        let mut exprs = vec![col(formatcp!(r#"^{MODE}.+$"#)), col(FATTY_ACID)];
        for item in key.export {
            if !item.visible {
                continue;
            }
            let expr = match item.column {
                Column::DeadTime => col(DEAD_TIME),
                Column::RetentionTime => col(formatcp!(r#"^{RETENTION_TIME}.+$"#)),
                Column::RetentionFactor => col(formatcp!(r#"^{RETENTION_FACTOR}.+$"#)),
                Column::SelectivityFactor => col(formatcp!(r#"^{SELECTIVITY_FACTOR}.+$"#)),
                Column::EquivalentChainLength => col(formatcp!(r#"^{EQUIVALENT_CHAIN_LENGTH}.+$"#)),
                Column::FractionalChainLength => col(formatcp!(r#"^{FRACTIONAL_CHAIN_LENGTH}.+$"#)),
                Column::Temperature => col(formatcp!(r#"^{TEMPERATURE}.+$"#)),
                Column::Mass => col(formatcp!(r#"^{MASS}.+$"#)),
            };
            exprs.push(expr);
        }
        lazy_frame = lazy_frame.select(exprs);
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
    pub(crate) export: &'a Export,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.mean_and_standard_deviation.ddof,
            export: &settings.export,
            precision: settings.precision.precision,
            significant: settings.precision.significant,
        }
    }
}

/// Flat export calculation value
type Value = HashedDataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .with_columns([
            col(MODE)
                .struct_()
                .field_by_name("*")
                .name()
                .prefix(formatcp!("{MODE}.")),
            col(FATTY_ACID).fatty_acid().delta(),
            col(DEAD_TIME).precision(key.precision, key.significant),
        ])
        .with_columns([
            format_array(
                col(RETENTION_TIME).struct_().field_by_name(ABSOLUTE),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{RETENTION_TIME}.{ABSOLUTE}.")),
            format_array(
                col(RETENTION_TIME).struct_().field_by_name(ADJUSTED),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{RETENTION_TIME}.{ADJUSTED}.")),
            format_array(
                col(RETENTION_TIME).struct_().field_by_name(RELATIVE),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{RETENTION_TIME}.{RELATIVE}.")),
        ])
        .with_columns([format_array(col(RETENTION_FACTOR), key, true)
            .name()
            .prefix(formatcp!("{RETENTION_FACTOR}."))])
        .with_columns([
            format_array(
                col(SELECTIVITY_FACTOR).struct_().field_by_name(FORWARD),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{SELECTIVITY_FACTOR}.{FORWARD}.")),
            format_array(
                col(SELECTIVITY_FACTOR).struct_().field_by_name(BACKWARD),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{SELECTIVITY_FACTOR}.{BACKWARD}.")),
        ])
        .with_columns([
            format_array(
                col(CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(EQUIVALENT_CHAIN_LENGTH),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{EQUIVALENT_CHAIN_LENGTH}.")),
            format_array(
                col(CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(FRACTIONAL_CHAIN_LENGTH),
                key,
                true,
            )
            .name()
            .prefix(formatcp!("{FRACTIONAL_CHAIN_LENGTH}.")),
        ])
        .with_columns([format_array(col(TEMPERATURE), key, true)
            .name()
            .prefix(formatcp!("{TEMPERATURE}."))])
        .with_columns([col(MASS)
            .struct_()
            .field_by_name("*")
            .name()
            .prefix(formatcp!("{MASS}."))]);
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
    let mut expr = as_struct(vec![
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
        (expr.clone().arr().std(key.ddof) / expr.clone().arr().mean())
            .percent(true)
            .precision(key.precision, key.significant)
            .alias(RELATIVE_STANDARD_DEVIATION),
    ]);
    if flatten {
        expr = expr.struct_().field_by_name("*");
    }
    expr
}

// fn _unnest<'a>(names: impl IntoIterator<Item = &'a str>, separator: Option<PlSmallStr>) -> Expr {
//     format_array(
//         col(SELECTIVITY_FACTOR).struct_().field_by_name(FORWARD),
//         key,
//         true,
//     )
//     .struct_()
//     .field_by_name("*")
//     .name()
//     .prefix(formatcp!("{SELECTIVITY_FACTOR}.{FORWARD}."))
// }

fn unnest<'a>(
    names: impl IntoIterator<Item = &'a str>,
    separator: Option<PlSmallStr>,
) -> PolarsResult<Expr> {
    let mut iter = names.into_iter();
    let Some(name) = iter.next() else {
        return Err(polars_err!(NoData: "Require at least one name"));
    };
    let mut expr = col(name);
    let prefix = name;
    for name in iter {
        expr = expr.struct_().field_by_name(name);
        if let Some(separator) = &separator {
            expr = expr.name().prefix(&format!("{prefix}{separator}"));
        }
    }
    Ok(expr)
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
