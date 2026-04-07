use crate::utils::hash::HashedDataFrame;
use anyhow::Result;
use metadata::{Metadata, polars::MetaDataFrame};
use polars_excel_writer::PolarsExcelWriter;
use std::borrow::Borrow;
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;

    #[instrument(skip(frame), err)]
    pub fn save(
        frame: &MetaDataFrame<impl Borrow<Metadata>, impl Borrow<HashedDataFrame>>,
        name: &str,
    ) -> Result<()> {
        let frame = MetaDataFrame::new(frame.meta.borrow(), frame.data.borrow());
        // Create a new Excel writer.
        let mut excel_writer = PolarsExcelWriter::new();
        // Write the dataframe to Excel.
        excel_writer.write_dataframe(frame.data)?;
        // Save the file to disk.
        excel_writer.save(name)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{XLSX, download};

    #[instrument(skip(frame), err)]
    pub fn save(
        frame: &MetaDataFrame<impl Borrow<Metadata>, impl Borrow<HashedDataFrame>>,
        name: &str,
    ) -> Result<()> {
        // let mut workbook = Workbook::new();
        // write(data_frame, workbook.add_worksheet())?;
        // let buffer = workbook.save_to_buffer()?;
        // if let Err(error) = download(&buffer, XLSX, name) {
        //     bail!("save: {error:?}");
        // }
        Ok(())
    }
}
