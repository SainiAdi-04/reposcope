use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_threshold_days: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum BranchState {
    Named(String),
    Detached(String),
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub short_hash: String,
    pub summary: String,
    pub author_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub project: Project,
    pub branch: BranchState,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub last_commit: Option<CommitInfo>,
    pub is_stale: bool,
    pub error: Option<String>,
}
