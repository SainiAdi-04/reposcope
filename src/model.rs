use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_threshold_days: Option<u32>,
}
