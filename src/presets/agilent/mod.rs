use crate::utils::hash::HashedMetaDataFrame;
use std::sync::LazyLock;

macro ron($name:literal) {
    LazyLock::new(|| {
        ron::de::from_bytes(include_bytes!($name)).expect(concat!("ron preset ", $name))
    })
}

pub(crate) static AGILENT: LazyLock<HashedMetaDataFrame> = ron!("Agilent[3.31].2026-05-13.ron");

// fn parse(bytes: &[u8]) -> Result<HashedMetaDataFrame> {
//     let frame = ron::de::from_bytes::<MetaDataFrame>(bytes)?;
//     let hmd_frame = MetaDataFrame {
//         meta: frame.meta,
//         data: HashedDataFrame::new(frame.data).unwrap(),
//     };
//     crate::export::ron::save(
//         &hmd_frame,
//         &format!("{}.cpft.ron", hmd_frame.meta.format(".")),
//     )?;
//     Ok(hmd_frame)
//     // Ok(MetaDataFrame {
//     //     meta: frame.meta,
//     //     data: HashedDataFrame::new(frame.data).unwrap(),
//     // })
// }

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
