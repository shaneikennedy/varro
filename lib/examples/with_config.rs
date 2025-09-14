use std::{path::Path, time::Duration};

use log::LevelFilter;
use varro::Varro;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    Varro::new(Path::new(".index"))
        .unwrap()
        .with_min_segment_size(1_000_000_000_000) // 1TB segments
        .with_max_buffer_size(1_000_000_000) // Flush when buffer his 1GB
        .with_compaction_frequency(Duration::from_secs(5)); // Try compaction every 5s
}
