use std::path::Path;

use anyhow::Result;
use log::{LevelFilter, info, warn};
use varro::{SearchOptions, Varro};

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let search_engine = Varro::new(Path::new("./.index"))?;
    if search_engine.index_size() == 0 {
        warn!("There are no documents in the index, try running the ingest exmaple first");
    }
    let opts = SearchOptions::new().with_include_documents(true);
    let results = search_engine.search("git and commit".into(), Some(opts));
    for (doc, score) in results {
        info!("Doc: {} with a score of: {}", doc.id(), score);
        let c = doc.get_field("contents".into()).unwrap();
        let mut c = c.contents();
        c.truncate(100);
        info!("Search result doc: {}, with contents: {}", doc.id(), c);
    }

    Ok(())
}
