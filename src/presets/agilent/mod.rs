use crate::utils::hash::{HashedDataFrame, HashedMetaDataFrame};
use anyhow::Result;
use metadata::polars::MetaDataFrame;
use std::sync::LazyLock;

macro ron($name:literal) {
    LazyLock::new(|| parse(include_bytes!($name)).expect(concat!("ron asset ", $name)))
}

fn parse(bytes: &[u8]) -> Result<HashedMetaDataFrame> {
    let frame = ron::de::from_bytes::<MetaDataFrame>(bytes)?;
    Ok(MetaDataFrame {
        meta: frame.meta,
        data: HashedDataFrame::new(frame.data).unwrap(),
    })
}

pub(crate) static AGILENT: LazyLock<HashedMetaDataFrame> = ron!("Agilent[0.2.0].2025-12-01.ron");

// macro ipc($name:literal) {
//     LazyLock::new(|| parse(include_bytes!($name)).expect(concat!("ipc asset ", $name)))
// }

// fn parse(bytes: &[u8]) -> Result<HashedMetaDataFrame> {
//     let mut reader = IpcReader::new(Cursor::new(bytes));
//     let meta = reader
//         .custom_metadata()?
//         .map(|meta| {
//             meta.iter()
//                 .map(|(key, value)| (key.to_string(), value.to_string()))
//                 .collect()
//         })
//         .unwrap_or_default();
//     let data = reader.finish()?;
//     Ok(MetaDataFrame {
//         meta,
//         data: HashedDataFrame::new(data)?,
//     })
// }

// pub(crate) static AGILENT: LazyLock<HashedMetaDataFrame> = ipc!("Agilent.ipc");

// pub(crate) static DEAD_TIME: LazyLock<HashedMetaDataFrame> = ipc!("DeadTime.ipc");
