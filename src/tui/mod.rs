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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Attention,
    Name,
    Recent,
    Dirty,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Attention => SortMode::Name,
            SortMode::Name => SortMode::Recent,
            SortMode::Recent => SortMode::Dirty,
            SortMode::Dirty => SortMode::Attention,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Attention => "attention",
            SortMode::Name => "name",
            SortMode::Recent => "recent",
            SortMode::Dirty => "dirty",
        }
    }
}

pub struct App {
    projects: Vec<Project>,
    pub statuses: Vec<RepoStatus>,
    pub selected: usize,
    pub search_query: String,
    pub is_searching: bool,
    pub sort_mode: SortMode,
    pub show_details: bool,
}

impl App {
    pub fn new(projects: Vec<Project>) -> Self {
        let mut app = Self {
            projects,
            statuses: Vec::new(),
            selected: 0,
            search_query: String::new(),
            is_searching: false,
            sort_mode: SortMode::Attention,
            show_details: false,
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.statuses = crate::scanner::concurrency::scan_all(self.projects.clone());
        self.clamp_selection();
    }

    pub fn visible_statuses(&self) -> Vec<&RepoStatus> {
        let query = self.search_query.trim().to_lowercase();
        let mut filtered: Vec<&RepoStatus> = self
            .statuses
            .iter()
            .filter(|s| {
                if query.is_empty() {
                    return true;
                }
                s.project.name.to_lowercase().contains(&query)
                    || s.branch.label().to_lowercase().contains(&query)
                    || s.last_commit
                        .as_ref()
                        .is_some_and(|c| c.summary.to_lowercase().contains(&query))
                    || s.error
                        .as_ref()
                        .is_some_and(|e| e.to_lowercase().contains(&query))
            })
            .collect();

        sort_statuses(&mut filtered, self.sort_mode);
        filtered
    }

    pub fn select_next(&mut self) {
        let count = self.visible_statuses().len();
        if count == 0 {
            return;
        }
        self.selected = (self.selected + 1).min(count - 1);
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.visible_statuses().len();
        if count > 0 {
            self.selected = count - 1;
        }
    }

    pub fn select_page_down(&mut self, page: usize) {
        let count = self.visible_statuses().len();
        if count == 0 {
            return;
        }
        self.selected = (self.selected + page).min(count - 1);
    }

    pub fn select_page_up(&mut self, page: usize) {
        self.selected = self.selected.saturating_sub(page);
    }

    pub fn selected_project_path(&self) -> Option<PathBuf> {
        let visible = self.visible_statuses();
        visible
            .get(self.selected)
            .map(|status| status.project.path.clone())
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.clamp_selection();
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn clamp_selection(&mut self) {
        let count = self.visible_statuses().len();
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(count - 1);
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

fn sort_statuses(statuses: &mut [&RepoStatus], mode: SortMode) {
    match mode {
        SortMode::Attention => {
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
        SortMode::Name => {
            statuses.sort_by(|left, right| left.project.name.cmp(&right.project.name));
        }
        SortMode::Recent => {
            statuses.sort_by(|left, right| {
                right
                    .last_commit_timestamp()
                    .cmp(&left.last_commit_timestamp())
                    .then_with(|| left.project.name.cmp(&right.project.name))
            });
        }
        SortMode::Dirty => {
            statuses.sort_by(|left, right| {
                right
                    .dirty_count()
                    .cmp(&left.dirty_count())
                    .then_with(|| status_rank(left).cmp(&status_rank(right)))
                    .then_with(|| left.project.name.cmp(&right.project.name))
            });
        }
    }
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
        let s1 = status("clean-old", 0, 0, 0, false, 10);
        let s2 = status("stale", 0, 0, 0, true, 40);
        let s3 = status("diverged", 0, 1, 0, false, 20);
        let s4 = status("dirty-new", 1, 0, 0, false, 30);
        let s5 = status("dirty-old", 1, 0, 0, false, 5);

        let mut refs = vec![&s1, &s2, &s3, &s4, &s5];
        sort_statuses(&mut refs, SortMode::Attention);

        let names: Vec<&str> = refs
            .iter()
            .map(|status| status.project.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["dirty-new", "dirty-old", "diverged", "clean-old", "stale"]
        );
    }

    #[test]
    fn visible_statuses_filters_by_query() {
        let s1 = status("alpha-repo", 0, 0, 0, false, 10);
        let s2 = status("beta-repo", 1, 0, 0, false, 20);
        let s3 = status("gamma-tool", 0, 0, 0, false, 30);

        let app = App {
            projects: Vec::new(),
            statuses: vec![s1, s2, s3],
            selected: 0,
            search_query: "repo".to_string(),
            is_searching: false,
            sort_mode: SortMode::Name,
            show_details: false,
        };

        let visible = app.visible_statuses();
        let names: Vec<&str> = visible.iter().map(|s| s.project.name.as_str()).collect();
        assert_eq!(names, vec!["alpha-repo", "beta-repo"]);
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
            has_upstream: true,
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
