use anyhow::Result;
use metadata::{MetaDataFrame, Metadata};
use polars::prelude::DataFrame;
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
pub fn save(name: &str, frame: MetaDataFrame<&Metadata, &mut DataFrame>) -> Result<()> {
    use polars::{io::SerWriter as _, prelude::*};

    let mut file = File::create(name)?;
    println!("{}", frame.data);
    CsvWriter::new(&mut file)
        .finish(
            &mut frame
                .data
                .clone()
                .lazy()
                .select([
                    col("Mode").struct_().field_by_index(0),
                    col("Mode").struct_().field_by_index(1),
                    col("FattyAcid").cast(DataType::String),
                    col("ChainLength").struct_().field_by_index(0),
                ])
                .collect()
                .unwrap(),
        )
        .unwrap();
    // MetaDataFrame::new(frame.meta.clone(), frame.data).write(file)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn save(name: &str, frame: MetaDataFrame<&Metadata, &mut DataFrame>) -> Result<()> {
    use anyhow::anyhow;
    use egui_ext::download;

    let mut bytes = Vec::new();
    MetaDataFrame::new(frame.meta.clone(), frame.data).write(&mut bytes)?;
    download(name, &bytes).map_err(|error| anyhow!(error))
}
