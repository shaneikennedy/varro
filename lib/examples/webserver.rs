use actix_web::{App, HttpServer, Responder, Result, get, web};
use log::{LevelFilter, info, warn};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fs, path::Path, sync::Arc};
use varro::{FileSystemType, Varro};

#[derive(Serialize)]
struct SearchResults {
    results: Vec<DocumentScore>,
}

type Document = (String, String);

#[derive(Serialize)]
struct DocumentScore {
    doc: Document,
    score: f64,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[get("/")]
async fn index(q: web::Query<SearchQuery>, varro: web::Data<Arc<Varro>>) -> Result<impl Responder> {
    info!("Got search request for: {}", q.q.clone());
    let results = varro.search(q.q.clone(), None);
    let mut results = results
        .map(|(doc, score)| {
            let doc = varro.retrieve(doc.id()).unwrap();

            DocumentScore {
                doc: (
                    doc.get_field("name".into()).unwrap().contents(),
                    // doc.get_field("contents".into()).unwrap().contents(),
                    "".into(),
                ),
                score,
            }
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Greater));

    Ok(web::Json(SearchResults { results }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    let search_engine =
        Arc::new(varro::Varro::new(Path::new("./.index"), FileSystemType::Local).unwrap());

    if search_engine.index_size() == 0 {
        warn!("There are no documents in the index, try running the ingest exmaple first");
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(search_engine.clone()))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
