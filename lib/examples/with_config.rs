use std::{path::Path, time::Duration};

use log::LevelFilter;
use varro::Varro;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    Varro::new(
        Path::new(".index"),
        varro::options::Options {
            filesystem: varro::options::FileSystemType::Local,
            flush: varro::options::FlushOptions {
                max_buffer_size: 1_000_000_000,
            },
            compaction: varro::options::CompactionOptions {
                min_segment_size: 1_000_000_000_000,
                compaction_frequency: Duration::from_secs(5),
            },
        },
    )
    .unwrap();
}
