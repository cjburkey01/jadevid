use core::ops::Range;
use ffmpeg_next::Rational;
use slotmap::SlotMap;
use std::path::PathBuf;

slotmap::new_key_type! { pub struct MediaKey; }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameNum(u64);

pub struct FrameSpan(Range<u64>);

pub struct MediaProject {
    pub fps: Rational,
    pub media: SlotMap<MediaKey, MediaReference>,
}

pub struct MediaReference {
    pub path: PathBuf,
}
