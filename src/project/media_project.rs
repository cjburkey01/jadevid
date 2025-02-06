use super::{JadeRational, MediaKey, MediaReference};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct MediaProject {
    pub fps: JadeRational,
    pub time_length: Duration,
    pub media: SlotMap<MediaKey, MediaReference>,
}
