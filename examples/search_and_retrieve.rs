use std::path::Path;

use anyhow::Result;
use log::{LevelFilter, error, info};
use varro::Varro;

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let search_engine = Varro::new(Path::new("./.index"))?;
    let res = search_engine.search("git and commit".into());
    for doc_score in res {
        info!(
            "Doc: {} with a score of: {}",
            doc_score.document_id, doc_score.score
        );
        let retrieved_doc = search_engine.retrieve(doc_score.document_id);
        match retrieved_doc {
            Some(d) => {
                let c = d.get_field("contents".into()).unwrap();
                let mut c = c.contents();
                c.truncate(100);
                info!("Search result doc: {}, with contents: {}", d.id(), c);
            }
            None => error!("Somethings wrong, couldn't find doc"),
        }
    }

    Ok(())
}
