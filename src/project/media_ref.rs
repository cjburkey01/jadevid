use super::JadeRational;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

slotmap::new_key_type! { pub struct MediaKey; }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub streams: Vec<MediaStream>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaLength {
    pub time_base_length: u64,
    pub time_base: JadeRational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicStreamInfo {
    pub index: usize,
    pub length: MediaLength,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoMediaStream {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioMediaStream {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaStream {
    Video(BasicStreamInfo, VideoMediaStream),
    Audio(BasicStreamInfo, AudioMediaStream),
}

impl MediaStream {
    #[allow(unused)]
    pub fn info(&self) -> BasicStreamInfo {
        match self {
            Self::Video(basic_info, _) | Self::Audio(basic_info, _) => basic_info.clone(),
        }
    }
}
