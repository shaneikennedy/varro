use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use anyhow::Result;
use bincode::config;
use log::{debug, error};
use uuid::Uuid;

use crate::{manifest::Manifest, segment::Segment};

pub(crate) struct SegmentCompactor {
    stop_signal: Arc<Mutex<bool>>,
    manifest: Arc<RwLock<Manifest>>,
    index_path: PathBuf,
    min_segment_size: Arc<Mutex<usize>>,
    compaction_freq: Arc<Mutex<Duration>>,
}

impl SegmentCompactor {
    pub fn new(
        stop_signal: Arc<Mutex<bool>>,
        manifest: Arc<RwLock<Manifest>>,
        index_path: PathBuf,
        min_segment_size: Arc<Mutex<usize>>,
        compaction_freq: Arc<Mutex<Duration>>,
    ) -> Self {
        SegmentCompactor {
            stop_signal,
            manifest,
            index_path,
            min_segment_size,
            compaction_freq,
        }
    }

    pub fn run(&self) -> Result<()> {
        while !*self.stop_signal.lock().unwrap() {
            let segments_guard = self.manifest.read().unwrap();
            debug!("Determine whate segments to compact");
            let segments_to_merge = segments_guard.segments.clone();
            drop(segments_guard);
            let min_size_guard = self.min_segment_size.lock().unwrap();
            let min_segment_size = *min_size_guard;
            drop(min_size_guard);
            let segments_to_merge = segments_to_merge
                .iter()
                .filter(|&(_, &size)| size <= min_segment_size)
                .clone();
            let mut merged_segment = Segment::new();
            if segments_to_merge.clone().count() > 1 {
                // Merge all small segments
                for (seg_id, _) in segments_to_merge.clone() {
                    let segment_file = format!("{seg_id}.seg");
                    let segment_path = self.index_path.join(&segment_file);
                    let contents = fs::read(&segment_path);
                    let segment = match contents {
                        Ok(c) => {
                            let config = config::standard();
                            let (decoded, _): (Segment, usize) =
                                bincode::decode_from_slice(&c[..], config).unwrap();
                            Some(decoded)
                        }
                        Err(_) => None,
                    };
                    match segment {
                        Some(s) => merged_segment.add_segment(s),
                        None => todo!(),
                    }
                }
                // write new merged segment
                let config = config::standard();
                let bytes = bincode::encode_to_vec(merged_segment, config).unwrap();
                let segment_id = Uuid::new_v4().to_string();
                let res = fs::write(self.index_path.join(segment_id.clone() + ".seg"), &bytes);
                match res {
                    Ok(_) => debug!("Successfully wrote new compacted segment {segment_id}"),
                    Err(_) => error!("Problem writing compacted segment"),
                }

                // update manifest to add new merge segment AND remove merged segments
                let mut manifest_guard = self.manifest.write().unwrap();
                manifest_guard.segments.insert(segment_id, bytes.len());
                for (seg_id, _) in segments_to_merge.clone() {
                    manifest_guard.segments.remove(seg_id);
                }
                drop(manifest_guard);

                // write the new manifest file
                let manifest_guard = self.manifest.read().unwrap();
                let bytes = bincode::encode_to_vec(&*manifest_guard, config).unwrap();
                let res = fs::write(self.index_path.join("manifest.varro"), bytes);
                match res {
                    Ok(_) => debug!("Successfully wrote new manifest"),
                    Err(_) => error!("Unable to write new manifest"),
                };
                drop(manifest_guard);

                // Cleanup merged segments
                for (seg_id, _) in segments_to_merge {
                    let res = fs::remove_file(self.index_path.join(format!("{seg_id}.seg")));
                    match res {
                        Ok(_) => debug!("Deleted {seg_id}.seg after compaction"),
                        Err(_) => error!("Problem deleting {seg_id}.seg after compaction"),
                    }
                }
            } else {
                debug!("No candidate segments for compaction.");
            }
            let sleep_guard = self.compaction_freq.lock().unwrap();
            let sleep = *sleep_guard;
            drop(sleep_guard);
            thread::sleep(sleep);
        }
        Ok(())
    }
}
