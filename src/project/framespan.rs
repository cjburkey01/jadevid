use super::FrameNum;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameSpan {
    pub from: FrameNum,
    pub to_excl: FrameNum,
}

impl From<FrameSpan> for Range<u64> {
    fn from(value: FrameSpan) -> Self {
        value.from.0..value.to_excl.0
    }
}

impl From<Range<u64>> for FrameSpan {
    fn from(value: Range<u64>) -> Self {
        Self {
            from: FrameNum(value.start),
            to_excl: FrameNum(value.end),
        }
    }
}
