use std::path::Path;

use crate::model::{BranchState, CommitInfo, Project, RepoStatus};

pub fn scan_one(path: &Path) -> RepoStatus {
    let project = Project {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        path: path.to_path_buf(),
        stale_threshold_days: None,
    };

    let repo = match git2::Repository::open(path) {
        Ok(r) => r,
        Err(e) => {
            return RepoStatus {
                project,
                error: Some(format!("open repo: {}", e.message())),
                branch: BranchState::Named(String::new()),
                ahead: 0,
                behind: 0,
                staged: 0,
                unstaged: 0,
                untracked: 0,
                last_commit: None,
                is_stale: false,
            }
        }
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(e) => {
            return RepoStatus {
                project,
                error: Some(format!("read HEAD: {}", e.message())),
                branch: BranchState::Named(String::new()),
                ahead: 0,
                behind: 0,
                staged: 0,
                unstaged: 0,
                untracked: 0,
                last_commit: None,
                is_stale: false,
            }
        }
    };

    let branch = if head.is_branch() {
        BranchState::Named(head.shorthand().unwrap_or("unknown").to_string())
    } else {
        BranchState::Detached(head.shorthand().unwrap_or("unknown").to_string())
    };

    let (ahead, behind) = compute_ahead_behind(&repo, &head);

    let last_commit = head.peel_to_commit().ok().map(|c| CommitInfo {
        short_hash: c.id().to_string()[..7].to_string(),
        summary: c.summary().unwrap_or("").to_string(),
        author_date: chrono::DateTime::from_timestamp(c.author().when().seconds(), 0)
            .unwrap_or_default(),
    });

    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(e) => {
            return RepoStatus {
                project,
                branch,
                ahead,
                behind,
                last_commit,
                error: Some(format!("read status: {}", e.message())),
                staged: 0,
                unstaged: 0,
                untracked: 0,
                is_stale: false,
            }
        }
    };

    let mut staged = 0u32;
    let mut unstaged = 0u32;
    let mut untracked = 0u32;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
        if s.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            unstaged += 1;
        }
        if s.contains(git2::Status::WT_NEW) {
            untracked += 1;
        }
    }

    RepoStatus {
        project,
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        last_commit,
        is_stale: false,
        error: None,
    }
}

fn compute_ahead_behind(repo: &git2::Repository, head: &git2::Reference) -> (u32, u32) {
    let local_oid = match head.peel_to_commit() {
        Ok(c) => c.id(),
        Err(_) => return (0, 0),
    };

    let branch_name = match head.shorthand() {
        Some(n) => n,
        None => return (0, 0),
    };

    let local_branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return (0, 0),
    };

    let upstream = match local_branch.upstream() {
        Ok(u) => u,
        Err(_) => return (0, 0),
    };

    let upstream_oid = match upstream.get().peel_to_commit() {
        Ok(c) => c.id(),
        Err(_) => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((a, b)) => (a as u32, b as u32),
        Err(_) => (0, 0),
    }
}
