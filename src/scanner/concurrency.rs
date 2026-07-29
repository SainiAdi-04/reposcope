use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;

use crate::model::{BranchState, Project, RepoStatus};

pub fn scan_all(projects: Vec<Project>) -> Vec<RepoStatus> {
    let project_count = projects.len();
    if project_count == 0 {
        return Vec::new();
    }

    let (tx, rx) = mpsc::channel();

    std::thread::scope(|scope| {
        for (index, project) in projects.into_iter().enumerate() {
            let tx = tx.clone();

            scope.spawn(move || {
                let fallback_project = project.clone();
                let status = catch_unwind(AssertUnwindSafe(|| super::scan_project(project)))
                    .unwrap_or_else(|_| error_status(fallback_project, "scanner panicked"));

                let _ = tx.send((index, status));
            });
        }
    });

    drop(tx);

    let mut statuses: Vec<Option<RepoStatus>> = vec![None; project_count];
    for (index, status) in rx {
        statuses[index] = Some(status);
    }

    statuses.into_iter().flatten().collect()
}

fn error_status(project: Project, message: &str) -> RepoStatus {
    RepoStatus {
        project,
        branch: BranchState::Named(String::new()),
        ahead: 0,
        behind: 0,
        has_upstream: false,
        staged: 0,
        unstaged: 0,
        untracked: 0,
        last_commit: None,
        is_stale: false,
        error: Some(message.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn scan_all_returns_empty_for_no_projects() {
        assert!(scan_all(Vec::new()).is_empty());
    }

    #[test]
    fn scan_all_keeps_results_ordered_and_captures_broken_repo() {
        let repo_root = std::env::current_dir().expect("test runs from crate root");
        let mut projects: Vec<Project> = (0..10)
            .map(|index| project(format!("repo-{index}"), repo_root.clone()))
            .collect();

        projects.push(project(
            "missing-repo",
            std::env::temp_dir().join("reposcope-phase3-missing-repo"),
        ));

        let statuses = scan_all(projects);

        assert_eq!(statuses.len(), 11);
        for (index, status) in statuses.iter().take(10).enumerate() {
            assert_eq!(status.project.name, format!("repo-{index}"));
            assert!(status.error.is_none());
        }

        let missing = statuses.last().expect("missing repo status exists");
        assert_eq!(missing.project.name, "missing-repo");
        assert!(missing.error.is_some());
    }

    fn project(name: impl Into<String>, path: PathBuf) -> Project {
        Project {
            name: name.into(),
            path,
            stale_threshold_days: None,
        }
    }
}
