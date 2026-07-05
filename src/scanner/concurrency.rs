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
        staged: 0,
        unstaged: 0,
        untracked: 0,
        last_commit: None,
        is_stale: false,
        error: Some(message.to_string()),
    }
}
