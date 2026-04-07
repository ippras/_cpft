use crate::{
    app::{
        computers::{
            distance::process::OUTPUT_SCHEMA as INPUT_SCHEMA, matches_schema, plot::IndexKey,
        },
        states::distance::Settings,
    },
    r#const::{
        DISTANCE, EQUIVALENT_CHAIN_LENGTH, FROM, MODE, ONSET_TEMPERATURE, SELECTIVITY_FACTOR,
        TEMPERATURE_STEP, TO,
    },
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::{
    Color32,
    emath::{Float, OrderedFloat},
    util::cache::{ComputerMut, FrameCache},
};
use egui_ext::color;
use egui_plot::PlotPoint;
use indexmap::IndexMap;
use lipid::prelude::*;
use polars::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    hash::{Hash, Hasher},
    iter::zip,
};

const COORDINATES: &str = "Coordinates";
const RANK: &str = "Rank";

/// Distance plot computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Distance plot computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        matches_schema(&key.frame.data_frame, &INPUT_SCHEMA)?;
        let lazy_frame = key.frame.data_frame.clone().lazy();
        // match key.settings.plot.axes {
        //     Axes {
        //         x: Axis::TemperatureStep,
        //         y: Axis::Alpha,
        //     } => temperature_step__alpha(lazy_frame),
        //     Axes {
        //         x: Axis::EquivalentChainLengths,
        //         y: Axis::Alpha,
        //     } => equivalent_chain_lengths__alpha(lazy_frame),
        //     _ => unimplemented!(),
        // }
        temperature_step_alpha(lazy_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        self.try_compute(key).expect("compute plot distance")
    }
}

/// Distance plot key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    // pub(crate) settings: &'a Settings,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            // filter: &settings.filter,
        }
    }
}

fn equivalent_chain_length_alpha(mut lazy_frame: LazyFrame) -> PolarsResult<Value> {
    // println!(
    //     "lazy_frame0: {}",
    //     lazy_frame
    //         .clone()
    //         .unnest([col(EQUIVALENT_CHAIN_LENGTH)])
    //         .collect()
    //         .unwrap()
    // );
    lazy_frame = lazy_frame
        .select([
            col(FATTY_ACID),
            col(MODE).struct_().field_by_name("*"),
            concat_arr(vec![
                col(EQUIVALENT_CHAIN_LENGTH)
                    .struct_()
                    .field_by_name(DISTANCE),
                col(SELECTIVITY_FACTOR),
            ])?
            .alias(COORDINATES),
        ])
        .with_column(col(FATTY_ACID).rank(Default::default(), None).alias(RANK))
        .unnest(cols([FATTY_ACID]), None);
    // .group_by([col(FATTY_ACID), col(ONSET_TEMPERATURE)])
    // .agg([as_struct(vec![col(TEMPERATURE_STEP), col(COORDINATES)])
    //     .sort(Default::default())])
    println!("lazy_frame1: {}", lazy_frame.clone().collect().unwrap());
    let data_frame = lazy_frame.collect()?;
    // Collect value
    let mut value = Value::default();
    for (((((from, to), onset_temperature), temperature_step), coordinates), rank) in zip(
        &data_frame[FROM].fatty_acid().fields()?,
        &data_frame[TO].fatty_acid().fields()?,
    )
    .zip(data_frame[ONSET_TEMPERATURE].f64()?.into_no_null_iter())
    .zip(data_frame[TEMPERATURE_STEP].f64()?.into_no_null_iter())
    .zip(data_frame[COORDINATES].array()?.into_no_null_iter())
    .zip(data_frame[RANK].u32()?.into_no_null_iter())
    {
        let Some(from) = from? else {
            polars_bail!(NoData: formatcp!("{FATTY_ACID}.{FROM}"));
        };
        let Some(to) = to? else {
            polars_bail!(NoData: formatcp!("{FATTY_ACID}.{TO}"));
        };
        value.fatty_acids.insert(rank, [from, to]);
        let coordinates = coordinates.f64()?;
        let Some(x) = coordinates.get(0) else {
            continue;
        };
        let Some(y) = coordinates.get(1) else {
            continue;
        };
        let point = PlotPoint::new(x, y);
        value
            .onset_temperature
            .entry((rank, onset_temperature.ord()))
            .or_default()
            .push(point);
        value
            .temperature_step
            .entry((rank, temperature_step.ord()))
            .or_default()
            .push(point);
        let entry = value.index.entry(IndexKey(point)).or_default();
        entry.insert(ONSET_TEMPERATURE, onset_temperature);
        entry.insert(TEMPERATURE_STEP, temperature_step);
    }
    Ok(value)
}

fn temperature_step_alpha(mut lazy_frame: LazyFrame) -> PolarsResult<Value> {
    lazy_frame = lazy_frame.select([
        col(FATTY_ACID),
        col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
        concat_arr(vec![
            col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
            col(SELECTIVITY_FACTOR),
        ])?
        .alias(COORDINATES),
    ]);
    println!("lazy_frame1: {}", lazy_frame.clone().collect().unwrap());
    let lazy_frame =
        lazy_frame.with_column(col(FATTY_ACID).rank(Default::default(), None).alias(RANK));
    println!("lazy_frame2: {}", lazy_frame.clone().collect().unwrap());
    let data_frame = lazy_frame.collect()?;
    let mut value = Value::default();
    let fatty_acid = data_frame[FATTY_ACID].struct_()?;
    for ((((from, to), onset_temperature), coordinates), rank) in zip(
        &fatty_acid.field_by_name(FROM)?.fatty_acid().fields()?,
        &fatty_acid.field_by_name(TO)?.fatty_acid().fields()?,
    )
    .zip(data_frame[ONSET_TEMPERATURE].f64()?.into_no_null_iter())
    .zip(data_frame[COORDINATES].array()?.into_no_null_iter())
    .zip(data_frame[RANK].u32()?.into_no_null_iter())
    {
        let Some(from) = from? else {
            polars_bail!(NoData: formatcp!("{FATTY_ACID}.{FROM}"));
        };
        let Some(to) = to? else {
            polars_bail!(NoData: formatcp!("{FATTY_ACID}.{TO}"));
        };
        value.fatty_acids.insert(rank, [from, to]);
        let coordinates = coordinates.f64()?;
        let Some(temperature_step) = coordinates.get(0) else {
            continue;
        };
        let Some(alpha) = coordinates.get(1) else {
            continue;
        };
        let point = PlotPoint::new(temperature_step, alpha);
        value
            .onset_temperature
            .entry((rank, onset_temperature.ord()))
            .or_default()
            .push(point);
        let entry = value.index.entry(IndexKey(point)).or_default();
        entry.insert(ONSET_TEMPERATURE, onset_temperature);
        entry.insert(TEMPERATURE_STEP, temperature_step);
        entry.insert(SELECTIVITY_FACTOR, alpha);

        // let mut points = Vec::new();
        // for coordinates in coordinates.array()?.into_no_null_iter() {
        //     let coordinates = coordinates.f64()?;
        //     let Some(x) = coordinates.get(0) else {
        //         continue;
        //     };
        //     let Some(y) = coordinates.get(1) else {
        //         continue;
        //     };
        //     points.push(PlotPoint::new(x, y));
        //     let entry = value
        //         .index
        //         .entry(IndexKey(PlotPoint::new(x, y)))
        //         .or_default();
        //     entry.insert(ONSET_TEMPERATURE.to_owned(), onset_temperature);
        //     entry.insert(TEMPERATURE_STEP.to_owned(), y);
        // }

        // value.lines.temperature_step.push(TemperatureStepLine {
        //     fatty_acids,
        //     onset_temperature,
        //     color: color(rank as _),
        //     points,
        // });
    }
    Ok(value)
}

// fn temperature_step_alpha(mut lazy_frame: LazyFrame) -> PolarsResult<Value> {
//     println!(
//         "lazy_frame0: {}",
//         lazy_frame
//             .clone()
//             .unnest([col(EQUIVALENT_CHAIN_LENGTH)])
//             .collect()
//             .unwrap()
//     );
//     lazy_frame = lazy_frame.select([
//         col(FATTY_ACID),
//         col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
//         concat_arr(vec![
//             col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
//             col(ALPHA),
//         ])?
//         .alias(COORDINATES),
//     ]);
//     println!("lazy_frame1: {}", lazy_frame.clone().collect().unwrap());
//     let lazy_frame = lazy_frame
//         .group_by([col(ONSET_TEMPERATURE), col(FATTY_ACID)])
//         .agg([col(COORDINATES).sort(Default::default())])
//         .with_column(
//             col(FATTY_ACID)
//                 .rank(Default::default(), None)
//                 .alias(RANK),
//         );
//     println!("lazy_frame2: {}", lazy_frame.clone().collect().unwrap());
//     let data_frame = lazy_frame.collect()?;
//     let mut value = Value::default();
//     let fatty_acid = data_frame[FATTY_ACID].struct_()?;
//     for ((((from, to), onset_temperature), coordinates), rank) in zip(
//         fatty_acid.field_by_name(FROM)?.fa().into_iter(),
//         fatty_acid.field_by_name(TO)?.fa().into_iter(),
//     )
//     .zip(data_frame[ONSET_TEMPERATURE].f64()?.into_no_null_iter())
//     .zip(data_frame[COORDINATES].array()?.into_no_null_iter())
//     .zip(data_frame[RANK].u32()?.into_no_null_iter())
//     {
//         let Some(from) = from else {
//             polars_bail!(NoData: "FattyAcid/From");
//         };
//         let Some(to) = to else {
//             polars_bail!(NoData: "FattyAcid/To");
//         };
//         let fatty_acids = [from, to];
//         let mut points = Vec::new();
//         for coordinates in coordinates.array()?.into_no_null_iter() {
//             let coordinates = coordinates.f64()?;
//             let Some(x) = coordinates.get(0) else {
//                 continue;
//             };
//             let Some(y) = coordinates.get(1) else {
//                 continue;
//             };
//             points.push(PlotPoint::new(x, y));
//             let entry = value
//                 .index
//                 .entry(IndexKey(PlotPoint::new(x, y)))
//                 .or_default();
//             entry.insert(ONSET_TEMPERATURE.to_owned(), onset_temperature);
//             entry.insert(TEMPERATURE_STEP.to_owned(), y);
//         }
//         value.lines.temperature_step.push(TemperatureStepLine {
//             fatty_acids,
//             onset_temperature,
//             color: color(rank as _),
//             points,
//         });
//     }
//     Ok(value)
// }

/// Distance plot value
#[derive(Clone, Default)]
pub(crate) struct Value {
    pub(crate) fatty_acids: BTreeMap<u32, [FattyAcid; 2]>,
    pub(crate) onset_temperature: BTreeMap<(u32, OrderedFloat<f64>), Vec<PlotPoint>>,
    pub(crate) temperature_step: BTreeMap<(u32, OrderedFloat<f64>), Vec<PlotPoint>>,
    pub(crate) index: HashMap<IndexKey, IndexMap<&'static str, f64>>,
}

// #[derive(Clone, Default)]
// pub(crate) struct Value {
//     pub(crate) lines: Lines,
//     pub(crate) index: HashMap<IndexKey, HashMap<String, f64>>,
// }

#[derive(Clone, Default)]
pub(crate) struct Lines {
    pub(crate) temperature_step: Vec<TemperatureStepLine>,
}

#[derive(Clone)]
pub(crate) struct TemperatureStepLine {
    pub(crate) fatty_acids: [FattyAcid; 2],
    pub(crate) color: Color32,
    pub(crate) onset_temperature: f64,
    pub(crate) points: Vec<PlotPoint>,
}

/// Index value
#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexValue {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Eq for IndexValue {}

impl Hash for IndexValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

impl PartialEq for IndexValue {
    fn eq(&self, other: &Self) -> bool {
        self.onset_temperature.ord() == other.onset_temperature.ord()
            && self.temperature_step.ord() == other.temperature_step.ord()
    }
}
