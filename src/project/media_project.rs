use super::{JadeRational, MediaInfo, MediaKey};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

#[derive(Serialize, Deserialize)]
pub struct MediaProject {
    pub fps: JadeRational,
    pub frame_count: u32,
    pub media: SlotMap<MediaKey, MediaInfo>,
}
