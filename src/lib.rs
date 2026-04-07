#![feature(decl_macro)]
#![feature(result_option_map_or_default)]
// #![feature(debug_closure_helpers)]
// #![feature(result_option_map_or_default)]
// #![feature(tuple_trait)]

pub use app::App;

mod app;
mod color;
mod r#const;
mod export;
mod localization;
mod presets;
mod utils;

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::{
        r#const::{
            ABSOLUTE, CHAIN_LENGTH, EQUIVALENT_CHAIN_LENGTH, MEAN, MODE, ONSET_TEMPERATURE,
            RETENTION_TIME, TEMPERATURE_STEP,
        },
        presets::AGILENT,
        utils::hash::{HashedDataFrame, HashedMetaDataFrame},
    };
    use lipid::{expr::ExprExt, prelude::*, r#trait::Atomic};
    use metadata::{Metadata, VERSION, polars::MetaDataFrame};
    use polars::prelude::*;
    use semver::Version;

    // Ok  Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}))
    // Err Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Parity': Boolean, 'Triple': Boolean}))}))
    #[test]
    fn rt() -> anyhow::Result<()> {
        println!("AGILENT.meta: {:?}", AGILENT.meta);
        println!("AGILENT.data: {:?}", AGILENT.data);
        // let onset_temperature = 70.0;
        // let temperature_step = 10.0;
        // let fa = C20C5C8C11C14.clone(); // 20:4Δ5c,8c,11c,14c
        // let condition = col("Mode")
        //     .struct_()
        //     .field_by_name("OnsetTemperature")
        //     .eq(lit(onset_temperature))
        //     .and(
        //         col("Mode")
        //             .struct_()
        //             .field_by_name("TemperatureStep")
        //             .eq(lit(temperature_step)),
        //     )
        //     .and(col("FattyAcid").fatty_acid().equal(fa));
        let condition = col(INDEX).eq(2468);
        let value1 = Option::<f64>::None;
        let value2 = Option::<f64>::None;
        let value3 = Option::<f64>::None;
        let value1 = Some(13.163);
        let value2 = Some(13.146);
        let value3 = Some(13.138);
        let mut lazy_frame = AGILENT.data.data_frame.clone().lazy();
        println!(
            "before: {:?}",
            lazy_frame
                .clone()
                .select([col(RETENTION_TIME).filter(condition.clone())])
                .collect()
                .unwrap()
        );
        lazy_frame = lazy_frame.with_columns([when(condition.clone())
            .then(
                concat_list(vec![
                    value1.map_or(col(RETENTION_TIME).arr().get(lit(0), true), lit),
                    value2.map_or(col(RETENTION_TIME).arr().get(lit(1), true), lit),
                    value3.map_or(col(RETENTION_TIME).arr().get(lit(2), true), lit),
                ])?
                .list()
                .to_array(3),
            )
            .otherwise(col(RETENTION_TIME))
            .alias(RETENTION_TIME)]);
        println!(
            "after: {:?}",
            lazy_frame
                .clone()
                .select([col(RETENTION_TIME).filter(condition.clone())])
                .collect()
                .unwrap()
        );

        let data = lazy_frame.collect()?;
        let mut meta = AGILENT.meta.clone();
        let mut version = meta[VERSION].parse::<Version>()?;
        version.patch += 1;
        meta.insert(VERSION.to_owned(), version.to_string());
        let name = format!("{}.ron", meta.format("."));
        let frame = MetaDataFrame::new(meta, HashedDataFrame::new(data)?);
        export::ron::save(&frame, &name)?;
        Ok(())
    }

    #[test]
    fn swap() -> anyhow::Result<()> {
        unsafe { std::env::set_var("POLARS_FMT_STR_LEN", "256") };
        unsafe { std::env::set_var("POLARS_TABLE_WIDTH", "256") };
        unsafe { std::env::set_var("POLARS_FMT_MAX_ROWS", "1024") };

        println!("AGILENT.meta: {:?}", AGILENT.meta);
        let mut lazy_frame = AGILENT.data.data_frame.clone().lazy();

        let i = 3148;
        let j = 3149;
        let predicate = col(INDEX).eq(i).or(col(INDEX).eq(j));
        println!(
            "before: {:?}",
            lazy_frame
                .clone()
                .filter(predicate.clone())
                .collect()
                .unwrap()
        );
        lazy_frame = lazy_frame
            .with_columns([when(col(INDEX).eq(lit(i)))
                .then(lit(j))
                .when(col(INDEX).eq(lit(j)))
                .then(lit(i))
                .otherwise(col(INDEX))
                .alias(INDEX)])
            .sort([INDEX], SortMultipleOptions::default());
        println!(
            "after: {:?}",
            lazy_frame.clone().filter(predicate).collect().unwrap()
        );

        let data = lazy_frame.collect()?;
        let mut meta = AGILENT.meta.clone();
        let mut version = meta[VERSION].parse::<Version>()?;
        version.patch += 1;
        meta.insert(VERSION.to_owned(), version.to_string());
        let name = format!("{}.ron", meta.format("."));
        let frame = MetaDataFrame::new(meta, HashedDataFrame::new(data)?);
        export::ron::save(&frame, &name)?;
        Ok(())
    }

    #[test]
    fn schema() -> anyhow::Result<()> {
        let mut lazy_frame = AGILENT.data.data_frame.clone().lazy();
        println!("schema: {:#?}", AGILENT.data.data_frame.schema());
        lazy_frame = lazy_frame.with_columns([col(RETENTION_TIME).list().to_array(3)]);
        println!("AGILENT: {:?}", lazy_frame.clone().collect().unwrap());
        let data = lazy_frame.collect()?;
        let frame = MetaDataFrame::new(AGILENT.meta.clone(), HashedDataFrame::new(data)?);
        export::ron::save(&frame, "name.temp.ron")?;
        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::{presets::AGILENT, utils::hash::HashedMetaDataFrame};
//     use lipid::{expr::ExprExt, prelude::*, r#trait::Atomic};
//     use metadata::{Metadata, polars::MetaDataFrame};
//     use polars::prelude::*;
//     use std::{fs::File, io::Cursor};
//
//     // Ok  Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}))
//     // Err Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Parity': Boolean, 'Triple': Boolean}))}))
//     #[test]
//     fn test() -> anyhow::Result<()> {
//         // let bytes = include_bytes!("presets/agilent/Agilent.ipc");
//         // let agilent = MetaDataFrame::read(Cursor::new(bytes)).unwrap();
//         // let bytes = include_bytes!("presets/agilent/DeadTime.ipc");
//         // let dead_time = MetaDataFrame::read(Cursor::new(bytes)).unwrap();
//         println!("AGILENT.meta: {:?}", AGILENT.meta);
//         println!("AGILENT.data: {:?}", AGILENT.data);
//         let lazy_frame = AGILENT
//             .data
//             .data_frame
//             .clone()
//             .lazy()
//             // .unnest(cols(["FattyAcid"]), Some(PlSmallStr::from_static(".")))
//             // .explode(cols(["FattyAcid.Unsaturated"]))
//             // .unnest(
//             //     cols(["FattyAcid.Unsaturated"]),
//             //     Some(PlSmallStr::from_static("."))
//             // )
//             // .filter(col("FattyAcid.Unsaturated.Isomerism").neq(lit(1)))
//             // "Isomerism" i8 1/-1
//             // "Unsaturation" u8 1
//             .select([
//                 col("Index"),
//                 col("Mode"),
//                 as_struct(vec![
//                     col(FATTY_ACID)
//                         .struct_()
//                         .field_by_name("Carbons")
//                         .alias(CARBON),
//                     col(FATTY_ACID)
//                         .struct_()
//                         .field_by_name("Unsaturated")
//                         .list()
//                         .eval(as_struct(vec![
//                             element()
//                                 .struct_()
//                                 .field_by_name(INDEX)
//                                 .cast(DataType::UInt8),
//                             element()
//                                 .struct_()
//                                 .field_by_name("Unsaturation")
//                                 .neq(1)
//                                 .alias(TRIPLE),
//                             element()
//                                 .struct_()
//                                 .field_by_name("Isomerism")
//                                 .eq(-1)
//                                 .alias(PARITY),
//                         ]))
//                         .alias(INDICES),
//                 ])
//                 .alias(FATTY_ACID),
//                 col("RetentionTime"),
//                 col("DeadTime"),
//             ]);
//         println!("AGILENT: {:?}", lazy_frame.clone().collect().unwrap());
//         let data = lazy_frame.collect()?;
//         let frame = MetaDataFrame::new(Metadata::default(), data);
//         export::ron::save(&frame, "name.ron")?;
//         // let mut data = agilent
//         //     .data
//         //     .lazy()
//         //     .join(
//         //         dead_time.data.lazy(),
//         //         [col("Mode").struct_().field_by_name("OnsetTemperature")],
//         //         [col("OnsetTemperature")],
//         //         JoinArgs::new(JoinType::Left),
//         //     )
//         //     .drop([col("OnsetTemperature")])
//         //     .collect()?;
//         // // println!("agilent: {}", data.collect()?);
//         // let frame = MetaDataFrame::new(&agilent.meta, &mut data);
//         // save("temp.ipc", frame)?;
//         Ok(())
//     }
// }
