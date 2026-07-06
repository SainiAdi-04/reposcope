use chrono::Utc;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::model::RepoStatus;

use super::App;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let [header_area, table_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);
    render_table(frame, table_area, app);
    render_footer(frame, footer_area);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let dirty = app
        .statuses
        .iter()
        .filter(|status| status.dirty_count() > 0)
        .count();
    let stale = app.statuses.iter().filter(|status| status.is_stale).count();
    let errors = app
        .statuses
        .iter()
        .filter(|status| status.error.is_some())
        .count();

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "reposcope",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "  {} repos  {} dirty  {} stale  {} errors",
                app.statuses.len(),
                dirty,
                stale,
                errors
            )),
        ]),
        Line::from(""),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = app.statuses.iter().map(status_row);

    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(26),
            Constraint::Min(18),
        ],
    )
    .header(
        Row::new(["repo", "branch", "sync", "changes", "last commit", "note"]).style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::default().borders(Borders::BOTTOM))
    .highlight_style(Style::default().bg(Color::DarkGray))
    .highlight_symbol("> ")
    .highlight_spacing(HighlightSpacing::Always);

    let mut state = TableState::default();
    if !app.statuses.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

fn status_row(status: &RepoStatus) -> Row<'static> {
    let style = row_style(status);
    let commit = status
        .last_commit
        .as_ref()
        .map(|commit| {
            format!(
                "{} {} ({})",
                commit.short_hash,
                commit.summary,
                relative_time(commit.author_date.timestamp())
            )
        })
        .unwrap_or_else(|| "-".to_string());

    let note = status
        .error
        .clone()
        .unwrap_or_else(|| status_note(status).to_string());

    Row::new(vec![
        Cell::from(status.project.name.clone()),
        Cell::from(status.branch.label().to_string()),
        Cell::from(sync_label(status)),
        Cell::from(change_label(status)),
        Cell::from(commit),
        Cell::from(note),
    ])
    .style(style)
}

fn row_style(status: &RepoStatus) -> Style {
    if status.error.is_some() {
        Style::default().fg(Color::Red)
    } else if status.dirty_count() > 0 {
        Style::default().fg(Color::Yellow)
    } else if status.divergence_count() > 0 {
        Style::default().fg(Color::Cyan)
    } else if status.is_stale {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green)
    }
}

fn sync_label(status: &RepoStatus) -> String {
    match (status.ahead, status.behind) {
        (0, 0) => "sync".to_string(),
        (ahead, 0) => format!("up {ahead}"),
        (0, behind) => format!("down {behind}"),
        (ahead, behind) => format!("up {ahead} down {behind}"),
    }
}

fn change_label(status: &RepoStatus) -> String {
    format!(
        "S:{} U:{} ?:{}",
        status.staged, status.unstaged, status.untracked
    )
}

fn status_note(status: &RepoStatus) -> &'static str {
    if status.is_stale {
        "stale"
    } else if status.dirty_count() > 0 {
        "dirty"
    } else if status.divergence_count() > 0 {
        "remote differs"
    } else {
        "clean"
    }
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = Paragraph::new("up/down navigate  r refresh  q quit")
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(footer, area);
}

fn relative_time(timestamp: i64) -> String {
    let Some(commit_time) = chrono::DateTime::from_timestamp(timestamp, 0) else {
        return "-".to_string();
    };

    let age = Utc::now().signed_duration_since(commit_time);
    if age.num_days() > 0 {
        format!("{}d ago", age.num_days())
    } else if age.num_hours() > 0 {
        format!("{}h ago", age.num_hours())
    } else {
        format!("{}m ago", age.num_minutes().max(0))
    }
}
