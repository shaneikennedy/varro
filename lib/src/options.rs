use std::time::Duration;

#[derive(Clone)]
pub enum FileSystemType {
    Local,
    Temp,
    #[cfg(feature = "s3")]
    S3,
}

#[derive(Clone)]
pub struct Options {
    pub compaction: CompactionOptions,
    pub flush: FlushOptions,
    pub filesystem: FileSystemType,
    pub semantic_search: SemanticSearchOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}

impl Options {
    pub fn new(
        compaction: Option<CompactionOptions>,
        flush: Option<FlushOptions>,
        filesystem: Option<FileSystemType>,
        semantic_search: Option<SemanticSearchOptions>,
    ) -> Self {
        Self {
            compaction: compaction.unwrap_or_default(),
            flush: flush.unwrap_or_default(),
            filesystem: filesystem.unwrap_or(FileSystemType::Local),
            semantic_search: semantic_search.unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub struct CompactionOptions {
    pub min_segment_size: usize,
    pub compaction_frequency: Duration,
}

impl Default for CompactionOptions {
    fn default() -> Self {
        Self::new(None, None)
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

#[derive(Clone)]
pub struct FlushOptions {
    pub max_buffer_size: usize,
}

impl Default for FlushOptions {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FlushOptions {
    pub fn new(max_buffer_size: Option<usize>) -> Self {
        Self {
            max_buffer_size: max_buffer_size.unwrap_or(50_000_000),
        }
    }
}

#[derive(Clone)]
pub struct SemanticSearchOptions {
    pub enabled: bool,
}

impl Default for SemanticSearchOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

impl SemanticSearchOptions {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}
