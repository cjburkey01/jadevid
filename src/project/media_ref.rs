use ffmpeg_next::Rational;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::JadeRational;

slotmap::new_key_type! { pub struct MediaKey; }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub streams: Vec<MediaStream>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaTime {
    pub length: u32,
    pub time_base: JadeRational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaStream {
    Video(MediaTime),
    Audio(MediaTime),
}

impl MediaStream {
    #[allow(unused)]
    pub fn time(&self) -> MediaTime {
        match self {
            Self::Video(media_time) | Self::Audio(media_time) => *media_time,
        }
    }
}
