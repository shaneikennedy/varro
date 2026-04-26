use std::time::Duration;

pub enum FileSystemType {
    Local,
    Temp,
    #[cfg(feature = "s3")]
    S3,
}

pub struct Options {
    pub compaction: CompactionOptions,
    pub flush: FlushOptions,
    pub filesystem: FileSystemType,
}

impl Default for Options {
    fn default() -> Self {
        Options::new(None, None, None)
    }
}

impl Options {
    pub fn new(
        compaction: Option<CompactionOptions>,
        flush: Option<FlushOptions>,
        filesystem: Option<FileSystemType>,
    ) -> Self {
        Self {
            compaction: compaction.unwrap_or_default(),
            flush: flush.unwrap_or_default(),
            filesystem: filesystem.unwrap_or(FileSystemType::Local),
        }
    }
}

pub struct CompactionOptions {
    pub min_segment_size: usize,
    pub compaction_frequency: Duration,
}

impl Default for CompactionOptions {
    fn default() -> Self {
        CompactionOptions::new(None, None)
    }
}

impl CompactionOptions {
    pub fn new(min_segment_size: Option<usize>, compaction_frequency: Option<Duration>) -> Self {
        Self {
            min_segment_size: min_segment_size.unwrap_or(64000000),
            compaction_frequency: compaction_frequency.unwrap_or(Duration::from_secs(2)),
        }
    }
}

pub struct FlushOptions {
    pub max_buffer_size: usize,
}

impl Default for FlushOptions {
    fn default() -> Self {
        FlushOptions::new(None)
    }
}

impl FlushOptions {
    pub fn new(max_buffer_size: Option<usize>) -> Self {
        Self {
            max_buffer_size: max_buffer_size.unwrap_or(50_000_000),
        }
    }
}
