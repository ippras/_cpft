use crate::utils::hash::HashedMetaDataFrame;
use std::sync::LazyLock;

macro ron($name:literal) {
    LazyLock::new(|| {
        ron::de::from_bytes(include_bytes!($name)).expect(concat!("ron preset ", $name))
    })
}

pub(crate) static AGILENT: LazyLock<HashedMetaDataFrame> = ron!("Agilent[3.32].2026-06-03.ron");

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
