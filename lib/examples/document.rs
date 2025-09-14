use log::{LevelFilter, info};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    info!("Document: {}", varro::Document::default().id());
}
