use std::path::Path;

use chrono::{Duration, Utc};

use crate::model::{BranchState, CommitInfo, Project, RepoStatus};

pub mod concurrency;

pub fn scan_one(path: &Path) -> RepoStatus {
    let project = Project {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        path: path.to_path_buf(),
        stale_threshold_days: None,
    };

    scan_project(project)
}

pub fn scan_project(project: Project) -> RepoStatus {
    let path = project.path.clone();

    let repo = match git2::Repository::open(&path) {
        Ok(r) => r,
        Err(e) => {
            return RepoStatus {
                project,
                error: Some(format!("open repo: {}", e.message())),
                branch: BranchState::Named(String::new()),
                ahead: 0,
                behind: 0,
                has_upstream: false,
                staged: 0,
                unstaged: 0,
                untracked: 0,
                last_commit: None,
                is_stale: false,
            }
        }
    };

    let (head, branch, ahead, behind, has_upstream, last_commit) = match repo.head() {
        Ok(h) => {
            let branch = if h.is_branch() {
                BranchState::Named(h.shorthand().unwrap_or("unknown").to_string())
            } else {
                BranchState::Detached(h.shorthand().unwrap_or("unknown").to_string())
            };
            let (ahead, behind, has_upstream) = compute_ahead_behind(&repo, &h);
            let last_commit = h.peel_to_commit().ok().map(|c| CommitInfo {
                short_hash: c.id().to_string()[..7].to_string(),
                summary: c.summary().unwrap_or("").to_string(),
                author_date: chrono::DateTime::from_timestamp(c.author().when().seconds(), 0)
                    .unwrap_or_default(),
            });
            (Some(h), branch, ahead, behind, has_upstream, last_commit)
        }
        Err(e) => {
            if let Some(target) = repo
                .find_reference("HEAD")
                .ok()
                .and_then(|r| r.symbolic_target().map(|s| s.to_string()))
            {
                let name = target
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&target)
                    .to_string();
                (None, BranchState::Named(name), 0, 0, false, None)
            } else {
                return RepoStatus {
                    project,
                    error: Some(format!("read HEAD: {}", e.message())),
                    branch: BranchState::Named(String::new()),
                    ahead: 0,
                    behind: 0,
                    has_upstream: false,
                    staged: 0,
                    unstaged: 0,
                    untracked: 0,
                    last_commit: None,
                    is_stale: false,
                };
            }
        }
    };

    let _ = head; // suppress unused warning if any

    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(e) => {
            return RepoStatus {
                project,
                branch,
                ahead,
                behind,
                has_upstream,
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

    let is_stale = is_stale(&last_commit, staged, unstaged, untracked, &project);

    RepoStatus {
        project,
        branch,
        ahead,
        behind,
        has_upstream,
        staged,
        unstaged,
        untracked,
        last_commit,
        is_stale,
        error: None,
    }
}

fn is_stale(
    last_commit: &Option<CommitInfo>,
    staged: u32,
    unstaged: u32,
    untracked: u32,
    project: &Project,
) -> bool {
    if staged > 0 || unstaged > 0 || untracked > 0 {
        return false;
    }

    let Some(commit) = last_commit else {
        return false;
    };

    let threshold_days = project.stale_threshold_days.unwrap_or(7);
    Utc::now().signed_duration_since(commit.author_date) > Duration::days(threshold_days.into())
}

fn compute_ahead_behind(repo: &git2::Repository, head: &git2::Reference) -> (u32, u32, bool) {
    let local_oid = match head.peel_to_commit() {
        Ok(c) => c.id(),
        Err(_) => return (0, 0, false),
    };

    let branch_name = match head.shorthand() {
        Some(n) => n,
        None => return (0, 0, false),
    };

    let local_branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return (0, 0, false),
    };

    let upstream = match local_branch.upstream() {
        Ok(u) => u,
        Err(_) => return (0, 0, false),
    };

    let upstream_oid = match upstream.get().peel_to_commit() {
        Ok(c) => c.id(),
        Err(_) => return (0, 0, true),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((a, b)) => (a as u32, b as u32, true),
        Err(_) => (0, 0, true),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use git2::{Repository, Signature, Time};

    use super::*;

    #[test]
    fn clean_repo_with_old_last_commit_is_stale() -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = create_repo_with_commit("old-clean", 10)?;
        let status = scan_project(project(repo_path, Some(7)));

        assert!(status.error.is_none());
        assert!(status.is_stale);

        Ok(())
    }

    #[test]
    fn dirty_repo_with_old_last_commit_is_not_stale() -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = create_repo_with_commit("old-dirty", 10)?;
        fs::write(repo_path.join("tracked.txt"), "changed")?;

        let status = scan_project(project(repo_path, Some(7)));

        assert!(status.error.is_none());
        assert!(!status.is_stale);
        assert_eq!(status.unstaged, 1);

        Ok(())
    }

    #[test]
    fn clean_repo_newer_than_threshold_is_not_stale() -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = create_repo_with_commit("fresh-clean", 1)?;
        let status = scan_project(project(repo_path, Some(7)));

        assert!(status.error.is_none());
        assert!(!status.is_stale);

        Ok(())
    }

    #[test]
    fn empty_repo_unborn_branch_scans_without_error() -> Result<(), Box<dyn std::error::Error>> {
        let path = unique_temp_path("unborn-repo")?;
        fs::create_dir_all(&path)?;
        Repository::init(&path)?;

        let status = scan_project(project(path, None));

        assert!(status.error.is_none());
        assert!(!status.has_upstream);
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
        assert!(status.last_commit.is_none());

        Ok(())
    }

    fn create_repo_with_commit(
        name: &str,
        age_days: i64,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = unique_temp_path(name)?;
        fs::create_dir_all(&path)?;

        let repo = Repository::init(&path)?;
        fs::write(path.join("tracked.txt"), "initial")?;

        let mut index = repo.index()?;
        index.add_path(Path::new("tracked.txt"))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let commit_time = Utc::now() - Duration::days(age_days);
        let signature = Signature::new(
            "Reposcope Test",
            "reposcope@example.invalid",
            &Time::new(commit_time.timestamp(), 0),
        )?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "test commit",
            &tree,
            &[],
        )?;

        drop(tree);
        drop(repo);

        Ok(path)
    }

    fn unique_temp_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        Ok(std::env::temp_dir().join(format!("reposcope-{name}-{nanos}")))
    }

    fn project(path: PathBuf, stale_threshold_days: Option<u32>) -> Project {
        Project {
            name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "test-repo".to_string()),
            path,
            stale_threshold_days,
        }
    }
}
