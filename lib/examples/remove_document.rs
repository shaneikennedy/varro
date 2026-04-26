use std::{
    fs::{self, read_to_string},
    path::Path,
};

use anyhow::{Context, Result};
use log::{LevelFilter, info};

pub fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let dir_contents = fs::read_dir("./documents")?;

    let mut files = Vec::new();
    for content in dir_contents {
        match content {
            Ok(c) => files.push(c.file_name()),
            Err(_) => panic!("something weird, entry in dir is not ok"),
        }
    }
    let search_engine = varro::Varro::new(
        Path::new("./.index"),
        varro::options::Options {
            filesystem: varro::options::FileSystemType::Local,
            flush: varro::options::FlushOptions::default(),
            compaction: varro::options::CompactionOptions::default(),
        },
    )?;
    assert!(
        search_engine.index_size() == 0,
        "this example won't work properly unless the index is empty"
    );
    // Ingest a document
    let file = files
        .first()
        .context("There should be atleast one file in the documents folder")
        .unwrap();
    let path = Path::new("./documents").join(file.clone());
    info!("path: {:#?}", path.clone());
    let contents = read_to_string(path)?;
    let mut doc = varro::Document::default();
    info!("Ingesting {}", file.to_str().unwrap());
    doc.add_field("name".into(), file.to_str().unwrap().to_string(), true);
    let mut debug_contents = contents.clone();
    debug_contents.truncate(150);
    info!(" with contents: {:#?}", debug_contents);
    doc.add_field("contents".into(), contents, true);
    search_engine.index(doc.clone())?;
    search_engine.flush()?;

    // Show that the document is retrievable and searchable
    search_engine
        .retrieve(doc.id())
        .context("Uh-oh! The doc isn't here after flushing")
        .unwrap();
    let results = search_engine.search(format!("name:'{}'", file.to_str().unwrap()), None);
    assert!(!results.collect::<Vec<_>>().is_empty());
    info!("Results found when searching for the title of the ingested document");
    // Now lets remove it
    search_engine.remove(&doc.id())?;
    let doc = search_engine.retrieve(doc.id());
    let results = search_engine.search(format!("name:'{}'", file.to_str().unwrap()), None);
    assert!(results.collect::<Vec<_>>().is_empty());
    assert!(doc.is_none());
    info!(
        "No results found when searching for the title of the ingested document, after removing it"
    );
    Ok(())
}
