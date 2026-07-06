pub mod input;
pub mod view;

use std::io;
use std::path::PathBuf;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::model::{Project, RepoStatus};

pub struct App {
    projects: Vec<Project>,
    pub statuses: Vec<RepoStatus>,
    pub selected: usize,
}

impl App {
    pub fn new(projects: Vec<Project>) -> Self {
        let mut app = Self {
            projects,
            statuses: Vec::new(),
            selected: 0,
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.statuses = crate::scanner::concurrency::scan_all(self.projects.clone());
        sort_statuses(&mut self.statuses);
        self.clamp_selection();
    }

    pub fn select_next(&mut self) {
        if self.statuses.is_empty() {
            return;
        }

        self.selected = (self.selected + 1).min(self.statuses.len() - 1);
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn selected_project_path(&self) -> Option<PathBuf> {
        self.statuses
            .get(self.selected)
            .map(|status| status.project.path.clone())
    }

    fn clamp_selection(&mut self) {
        if self.statuses.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.statuses.len() - 1);
        }
    }
}

pub fn run(projects: Vec<Project>) -> io::Result<Option<PathBuf>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal, App::new(projects));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<Option<PathBuf>> {
    loop {
        terminal.draw(|frame| view::render(frame, &app))?;

        match input::handle_event(&mut app)? {
            super::tui::input::Action::Continue => {}
            super::tui::input::Action::Quit => return Ok(None),
            super::tui::input::Action::QuitWithSelectedPath => {
                return Ok(app.selected_project_path())
            }
        }
    }
}

fn sort_statuses(statuses: &mut [RepoStatus]) {
    statuses.sort_by(|left, right| {
        status_rank(left)
            .cmp(&status_rank(right))
            .then_with(|| {
                right
                    .last_commit_timestamp()
                    .cmp(&left.last_commit_timestamp())
            })
            .then_with(|| left.project.name.cmp(&right.project.name))
    });
}

fn status_rank(status: &RepoStatus) -> u8 {
    if status.error.is_some() {
        0
    } else if status.dirty_count() > 0 {
        1
    } else if status.divergence_count() > 0 {
        2
    } else if status.is_stale {
        4
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::model::{BranchState, CommitInfo};

    use super::*;

    #[test]
    fn sort_statuses_prioritizes_attention_then_recency() {
        let mut statuses = vec![
            status("clean-old", 0, 0, 0, false, 10),
            status("stale", 0, 0, 0, true, 40),
            status("diverged", 0, 1, 0, false, 20),
            status("dirty-new", 1, 0, 0, false, 30),
            status("dirty-old", 1, 0, 0, false, 5),
        ];

        sort_statuses(&mut statuses);

        let names: Vec<&str> = statuses
            .iter()
            .map(|status| status.project.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["dirty-new", "dirty-old", "diverged", "clean-old", "stale"]
        );
    }

    fn status(
        name: &str,
        unstaged: u32,
        behind: u32,
        ahead: u32,
        is_stale: bool,
        timestamp: i64,
    ) -> RepoStatus {
        RepoStatus {
            project: Project {
                name: name.to_string(),
                path: name.into(),
                stale_threshold_days: None,
            },
            branch: BranchState::Named("main".to_string()),
            ahead,
            behind,
            staged: 0,
            unstaged,
            untracked: 0,
            last_commit: Some(CommitInfo {
                short_hash: "abc1234".to_string(),
                summary: "summary".to_string(),
                author_date: Utc.timestamp_opt(timestamp, 0).unwrap(),
            }),
            is_stale,
            error: None,
        }
    }
}
