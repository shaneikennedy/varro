use std::path::Path;

use anyhow::Result;
use log::{LevelFilter, warn};
use varro::{FileSystemType, SearchOperator, SearchOptions, Varro};

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let search_engine = Varro::new(Path::new("./.index"), FileSystemType::Local)?;
    if search_engine.index_size() == 0 {
        warn!("There are no documents in the index, try running the ingest exmaple first");
    }
    let opts = SearchOptions::new()
        .with_include_documents(true)
        .with_search_operator(SearchOperator::AND);
    let results = search_engine.search("git gorilla".into(), Some(opts));
    assert_eq!(
        results.count(),
        0,
        "surely there are no docs that talk about both git AND gorillas"
    );
    Ok(())
}
