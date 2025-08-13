use log::{LevelFilter, info};

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Document: {}", varro::Document::new().id());
}
