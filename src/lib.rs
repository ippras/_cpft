#![feature(debug_closure_helpers)]
#![feature(decl_macro)]
#![feature(tuple_trait)]

pub use app::App;

mod app;
mod r#const;
mod export;
mod localization;
mod presets;
mod utils;

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        r#const::{ABSOLUTE, CHAIN_LENGTH, EQUIVALENT_CHAIN_LENGTH, MEAN, MODE, RETENTION_TIME},
        presets::AGILENT,
        utils::hash::{HashedDataFrame, HashedMetaDataFrame},
    };
    use lipid::{expr::ExprExt, prelude::*, r#trait::Atomic};
    use metadata::{Metadata, polars::MetaDataFrame};
    use polars::prelude::*;

    // Ok  Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}))
    // Err Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Parity': Boolean, 'Triple': Boolean}))}))
    // 2.4: 80.0 5.0 C22DC13
    // 2.5: 80.0 7.0 C23
    // 2.6: 90.0 8.0 C22DC13
    // 2.7: 130.0 9.0 C23
    // 2.8: 130.0 10.0 C22DC13
    // 2.9: 130.0 10.0 C23
    // 2.10: 60.0 3.0 C20DC11
    // 2.10: 120.0 9.0 C22DC13
    // 2.11: 60.0 5.0 C18DC9DC12DC15
    //
    // 2.12: 100.0 7.0 C18DC9DC12DC15
    // 2.13: 70.0 8.0 C18DC9DC12
    // 2.14: 110.0 6.0 C18DT9
    #[test]
    fn rt() -> anyhow::Result<()> {
        println!("AGILENT.meta: {:?}", AGILENT.meta);
        println!("AGILENT.data: {:?}", AGILENT.data);
        let onset_temperature = 110.0;
        let temperature_step = 6.0;
        let fa = C18DT9.clone();
        // let value1 = Option::<f64>::None;
        // let value2 = Option::<f64>::None;
        // let value3 = Option::<f64>::None;
        let value1 = Some(16.884);
        let value2 = Some(16.852);
        let value3 = Some(16.842);
        let mut lazy_frame = AGILENT.data.data_frame.clone().lazy();
        let condition = col("Mode")
            .struct_()
            .field_by_name("OnsetTemperature")
            .eq(lit(onset_temperature))
            .and(
                col("Mode")
                    .struct_()
                    .field_by_name("TemperatureStep")
                    .eq(lit(temperature_step)),
            )
            .and(col("FattyAcid").fatty_acid().equal(fa));
        println!(
            "before: {:?}",
            lazy_frame
                .clone()
                .select([col("RetentionTime").filter(condition.clone())])
                .collect()
                .unwrap()
        );
        lazy_frame = lazy_frame.with_columns([when(condition.clone())
            .then(concat_list([
                value1.map_or(col("RetentionTime").list().get(lit(0), true), lit),
                value2.map_or(col("RetentionTime").list().get(lit(1), true), lit),
                value3.map_or(col("RetentionTime").list().get(lit(2), true), lit),
            ])?)
            .otherwise(col("RetentionTime"))
            .alias("RetentionTime")]);
        println!(
            "after: {:?}",
            lazy_frame
                .clone()
                .select([col("RetentionTime").filter(condition.clone())])
                .collect()
                .unwrap()
        );
        println!("AGILENT: {:?}", lazy_frame.clone().collect().unwrap());
        let data = lazy_frame.collect()?;
        let frame = MetaDataFrame::new(AGILENT.meta.clone(), HashedDataFrame::new(data)?);
        export::ron::save(&frame, "name.temp.ron")?;
        Ok(())
    }

    // (10.969+10.966)/2=10.9675
    // (10.967+10.944+10.941)/3=10.950666666666666667
    #[test]
    fn sort() -> anyhow::Result<()> {
        unsafe { std::env::set_var("POLARS_FMT_STR_LEN", "256") };
        unsafe { std::env::set_var("POLARS_TABLE_WIDTH", "256") };
        unsafe { std::env::set_var("POLARS_FMT_MAX_ROWS", "1024") };

        println!("AGILENT.meta: {:?}", AGILENT.meta);
        println!("AGILENT.data: {:?}", AGILENT.data);
        let sort_options = SortMultipleOptions::new()
            .with_maintain_order(true)
            .with_nulls_last(false)
            .with_order_descending(false);
        let mut lazy_frame = AGILENT.data.data_frame.clone().lazy();
        println!("before: {:?}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.sort([MODE], sort_options.clone()).select([all()
            .as_expr()
            .sort_by(&[col(RETENTION_TIME).list().mean()], sort_options)
            .over([col(MODE)])?]);
        // println!("after: {:?}", lazy_frame.clone().collect().unwrap());
        // println!("AGILENT: {:?}", lazy_frame.clone().collect().unwrap());
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
