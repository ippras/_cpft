#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

use crate::utils::hash::HashedDataFrame;
use anyhow::Result;
use metadata::{Metadata, polars::MetaDataFrame};
use std::borrow::Borrow;
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::{fs::File, io::Write};

    #[instrument(skip(frame), err)]
    pub fn save(
        frame: &MetaDataFrame<impl Borrow<Metadata>, impl Borrow<HashedDataFrame>>,
        name: &str,
    ) -> Result<()> {
        let frame = MetaDataFrame::new(frame.meta.borrow(), frame.data.borrow());
        let mut file = File::create(name)?;
        let serialized = frame.data.to_string();
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};

    #[instrument(skip(frame), err)]
    pub fn save(
        mut frame: &MetaDataFrame<impl Borrow<Metadata>, impl Borrow<HashedDataFrame>>,
        name: &str,
    ) -> Result<()> {
        // let mut bytes = Vec::new();
        // let mut writer = CsvWriter::new(&mut bytes);
        // writer.finish(data_frame.borrow_mut())?;
        // if let Err(error) = download(&bytes, NONE, name) {
        //     bail!("save csv: {error:?}");
        // }
        Ok(())
    }
}
