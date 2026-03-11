#![feature(debug_closure_helpers)]
#![feature(decl_macro)]
#![feature(tuple_trait)]

pub use app::App;

mod app;
mod r#const;
mod export;
mod localization;
mod presets;
mod special;
mod utils;

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        presets::AGILENT,
        utils::hash::{HashedDataFrame, HashedMetaDataFrame},
    };
    use lipid::{expr::ExprExt, prelude::*, r#trait::Atomic};
    use metadata::{Metadata, polars::MetaDataFrame};
    use polars::prelude::*;
    use std::{fs::File, io::Cursor};

    // Ok  Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}))
    // Err Some(Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Parity': Boolean, 'Triple': Boolean}))}))
    #[test]
    fn test() -> anyhow::Result<()> {
        println!("AGILENT.meta: {:?}", AGILENT.meta);
        println!("AGILENT.data: {:?}", AGILENT.data);
        let onset_temperature = 60.0;
        let temperature_step = 3.0;
        let fa = C20DC11.clone(); // 20:1Δ11c
        // let value1 = 14.148;
        // let value2 = 0.0;
        let value3 = 47.239;
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
                col("RetentionTime").list().get(lit(0), true),
                // lit(value1),
                col("RetentionTime").list().get(lit(1), true),
                lit(value3),
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
