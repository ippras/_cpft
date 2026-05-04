use crate::{
    app::{computers::plot::IndexKey, states::source::Settings},
    r#const::{ONSET_TEMPERATURE, *},
    utils::hash::HashedDataFrame,
};
use egui::{
    emath::Float,
    util::cache::{ComputerMut, FrameCache},
};
use egui_plot::PlotPoint;
use lipid::prelude::*;
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

pub(crate) const POINTS: &str = "Points";

/// Source plot computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Source plot computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = lazy_frame.filter(col(FILTER));
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        // Pack
        pack(lazy_frame.collect()?, key)

        // // let lazy_frame = lazy_frame.rank()
        // let lazy_frame = lazy_frame
        //     .clone()
        //     .group_by([col(FATTY_ACID), col(ONSET_TEMPERATURE)])
        //     .agg([
        //         col(TEMPERATURE_STEP),
        //         col(RETENTION_TIME),
        //         col(EQUIVALENT_CHAIN_LENGTH),
        //     ])
        //     .group_by([col(FATTY_ACID)])
        //     .agg([
        //         col(ONSET_TEMPERATURE),
        //         col(TEMPERATURE_STEP),
        //         col(RETENTION_TIME),
        //         col(EQUIVALENT_CHAIN_LENGTH),
        //     ]);

        // let lazy_frame2 = lazy_frame
        //     .clone()
        //     .group_by([col(FATTY_ACID), col(TEMPERATURE_STEP)])
        //     .agg([
        //         col(ONSET_TEMPERATURE),
        //         col(RETENTION_TIME),
        //         col(EQUIVALENT_CHAIN_LENGTH),
        //     ])
        //     .group_by([col(FATTY_ACID)])
        //     .agg([
        //         col(TEMPERATURE_STEP),
        //         col(ONSET_TEMPERATURE),
        //         col(RETENTION_TIME),
        //         col(EQUIVALENT_CHAIN_LENGTH),
        //     ])
        //     .sort([FATTY_ACID], Default::default());
        // println!(
        //     "lazy_frame2 by FattyAcid/TemperatureStep/OnsetTemperature: {}",
        //     lazy_frame2.clone().collect().unwrap()
        // );
        // lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        self.try_compute(key).expect("compute plot source")
    }
}

/// Source plot key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) precision: usize,
    pub(crate) logarithmic: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            precision: settings.precision,
            logarithmic: settings.logarithmic,
        }
    }
}

// impl Hash for Key<'_> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.settings.ddof.hash(state);
//         self.settings.logarithmic.hash(state);
//         self.settings.filter.hash(state);
//         self.settings.radius_of_points.hash(state);
//     }
// }

/// Source plot value
#[derive(Clone, Default)]
pub(crate) struct Value {
    pub(crate) lines: Lines,
    pub(crate) index: HashMap<IndexKey, HashSet<PointValue>>,
}

#[derive(Clone, Default)]
pub(crate) struct Lines {
    pub(crate) temperature_step: Vec<TemperatureStepLine>,
    // pub(crate) onset_temperature: Vec<OnsetTemperatureLine>,
}

#[derive(Clone)]
pub(crate) struct TemperatureStepLine {
    pub(crate) fatty_acid: FattyAcid,
    pub(crate) onset_temperature: f64,
    pub(crate) points: Vec<PlotPoint>,
}

// #[derive(Clone)]
// pub(crate) struct OnsetTemperatureLine {
//     pub(crate) fatty_acid: FattyAcid,
//     pub(crate) temperature_step: f64,
//     pub(crate) points: Vec<PlotPoint>,
// }

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointValue {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Eq for PointValue {}

impl Hash for PointValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

impl PartialEq for PointValue {
    fn eq(&self, other: &Self) -> bool {
        self.onset_temperature.ord() == other.onset_temperature.ord()
            && self.temperature_step.ord() == other.temperature_step.ord()
    }
}

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // println!(
    //     "lazy_frame: {}",
    //     lazy_frame
    //         .clone()
    //         .select([col(CHAIN_LENGTH).struct_().field_by_name("*")])
    //         .collect()?
    // );

    lazy_frame = lazy_frame.select([
        col(MODE).struct_().field_by_name(ONSET_TEMPERATURE),
        col(MODE).struct_().field_by_name(TEMPERATURE_STEP),
        col(FATTY_ACID),
        col(RETENTION_TIME)
            .struct_()
            .field_by_name(ABSOLUTE)
            .struct_()
            .field_by_name(MEAN)
            .alias(RETENTION_TIME),
        col(CHAIN_LENGTH)
            .struct_()
            .field_by_name(EQUIVALENT_CHAIN_LENGTH),
    ]);
    // println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.select([
        col(ONSET_TEMPERATURE),
        col(TEMPERATURE_STEP),
        col(FATTY_ACID),
        concat_arr(vec![col(RETENTION_TIME), col(EQUIVALENT_CHAIN_LENGTH)])?.alias(POINTS),
    ]);
    let lazy_frame1 = lazy_frame
        .group_by([col(FATTY_ACID), col(ONSET_TEMPERATURE)])
        .agg([col(TEMPERATURE_STEP), col(POINTS)]);
    Ok(lazy_frame)
}

fn pack(mut data_frame: DataFrame, key: Key) -> PolarsResult<Value> {
    let mut value = Value::default();
    for (((fatty_acid, onset_temperature), temperature_steps), points) in data_frame[FATTY_ACID]
        .fatty_acid()
        .fields()?
        .into_iter()
        .zip(data_frame[ONSET_TEMPERATURE].f64()?.into_no_null_iter())
        .zip(data_frame[TEMPERATURE_STEP].list()?.into_no_null_iter())
        .zip(data_frame[POINTS].list()?.into_no_null_iter())
    {
        let fatty_acid = fatty_acid.unwrap(); // TODO
        let mut line_points = Vec::new();
        for (temperature_step, points) in temperature_steps
            .f64()?
            .into_no_null_iter()
            .zip(points.array()?.into_no_null_iter())
        {
            let points = points.f64()?;
            let Some(x) = points.get(0) else {
                continue;
            };
            let Some(y) = points.get(1) else {
                continue;
            };
            line_points.push(PlotPoint::new(x, y));
            value
                .index
                .entry(IndexKey(PlotPoint::new(x, y)))
                .or_default()
                .insert(PointValue {
                    onset_temperature,
                    temperature_step,
                });
        }
        value.lines.temperature_step.push(TemperatureStepLine {
            fatty_acid,
            onset_temperature,
            points: line_points,
        });
        // value.onset_temperatures.push(OnsetTemperaturePoints {
        //     fatty_acid,
        //     onset_temperature,
        //     points: line_points,
        // });
        // let index = Int64Chunked::from_vec(PlSmallStr::EMPTY, vec![1]);
        // let z = points.array()?.array_get(&index, false)?;
    }
    Ok(value)
}
