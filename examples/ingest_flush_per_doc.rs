use std::{
    fs::{self, read_to_string},
    path::Path,
};

use anyhow::Result;
use log::{LevelFilter, info};

pub fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    let dir_contents = fs::read_dir("./documents")?;

    let mut files = Vec::new();
    for content in dir_contents {
        match content {
            Ok(c) => files.push(c.file_name()),
            Err(_) => panic!("something weird, entry in dir is not ok"),
        }
    }
    let search_engine = varro::Varro::new(Path::new("./.index"))?;
    for file in files {
        let path = Path::new("./documents").join(file.clone());
        info!("path: {:#?}", path.clone());
        let contents = read_to_string(path)?;
        let mut doc = varro::Document::default();
        info!("Ingesting {}", file.to_str().unwrap());
        doc.add_field("name".into(), file.to_str().unwrap().to_string(), false);
        let mut debug_contents = contents.clone();
        debug_contents.truncate(150);
        info!(" with contents: {:#?}", debug_contents);
        doc.add_field("contents".into(), contents, false);
        search_engine.index(doc)?;
        search_engine.flush()?;
    }

    Ok(())
}
