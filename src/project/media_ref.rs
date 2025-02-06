use serde::{Deserialize, Serialize};
use std::path::PathBuf;

slotmap::new_key_type! { pub struct MediaKey; }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaReference {
    pub path: PathBuf,
}
