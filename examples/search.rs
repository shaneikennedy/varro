use std::{fs::read_to_string, path::Path};

use anyhow::Result;

fn main() -> Result<()> {
    let contents = read_to_string("./documents/git-intro.md")?;
    let search_engine = varro::Varro::new(Path::new("./.index"))?;
    let mut doc = varro::Document::new();
    doc.add_field("name".into(), "git-intro".into(), false);
    doc.add_field("contents".into(), contents, false);
    search_engine.index(doc)?;
    search_engine.flush()?;

    let res = search_engine.search("git and commit".into());
    println!("Found 'git' in docs: {res:#?}");

    for doc_id in res {
        let retrieved_doc = search_engine.retrieve(doc_id);
        match retrieved_doc {
            Some(d) => {
                let c = d.get_field("contents".into()).unwrap();
                let mut c = c.contents();
                c.truncate(100);
                println!("Search result doc: {}, with contents: {}", d.id(), c);
                println!(
                    "Search result doc contains search query: {}",
                    c.contains("git")
                );
            }
            None => println!("Somethings wrong, couldn't find doc"),
        }
    }

    Ok(())
}
