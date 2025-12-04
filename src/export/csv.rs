#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

use anyhow::Result;
use polars::prelude::*;
use std::borrow::BorrowMut;
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::fs::File;

    #[instrument(skip(data_frame), err)]
    pub fn save(mut data_frame: impl BorrowMut<DataFrame>, name: &str) -> Result<()> {
        let mut file = File::create(name)?;
        CsvWriter::new(&mut file).finish(data_frame.borrow_mut())?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};

    #[instrument(skip(data_frame), err)]
    pub fn save(mut data_frame: impl BorrowMut<DataFrame>, name: &str) -> Result<()> {
        let mut bytes = Vec::new();
        let mut writer = CsvWriter::new(&mut bytes);
        writer.finish(data_frame.borrow_mut())?;
        if let Err(error) = download(&bytes, NONE, name) {
            bail!("save csv: {error:?}");
        }
        Ok(())
    }
}
