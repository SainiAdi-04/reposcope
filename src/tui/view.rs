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
    let visible = app.visible_statuses();
    let main_constraints = if app.show_details {
        vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(5),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
        ]
    };

    let chunks = Layout::vertical(main_constraints).split(frame.area());

    render_header(frame, chunks[0], app, visible.len());
    render_table(frame, chunks[1], app, &visible);
    if app.show_details {
        render_details(frame, chunks[2], app, &visible);
        render_footer(frame, chunks[3], app);
    } else {
        render_footer(frame, chunks[2], app);
    }
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App, visible_count: usize) {
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

    let details_state = if app.show_details { "on" } else { "off" };
    let total = app.statuses.len();
    let count_str = if visible_count != total {
        format!("{visible_count}/{total} repos")
    } else {
        format!("{total} repos")
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "reposcope",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "  {}  {} dirty  {} stale  {} errors",
                count_str, dirty, stale, errors
            )),
            Span::styled(
                format!("  [sort: {} (s)]", app.sort_mode.label()),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("  [details: {} (d)]", details_state),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(""),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame<'_>, area: Rect, app: &App, visible: &[&RepoStatus]) {
    let rows = visible.iter().map(|s| status_row(s));

    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(16),
            Constraint::Length(12),
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
    if !visible.is_empty() {
        state.select(Some(app.selected.min(visible.len() - 1)));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

fn render_details(frame: &mut Frame<'_>, area: Rect, app: &App, visible: &[&RepoStatus]) {
    let content = if visible.is_empty() {
        vec![Line::from("No repository selected")]
    } else {
        let index = app.selected.min(visible.len() - 1);
        let s = visible[index];
        let commit_str = s
            .last_commit
            .as_ref()
            .map(|c| {
                format!(
                    "{} - {} ({})",
                    c.short_hash,
                    c.summary,
                    c.author_date.format("%Y-%m-%d %H:%M:%S UTC")
                )
            })
            .unwrap_or_else(|| "No commits yet".to_string());

        let threshold = s.project.stale_threshold_days.unwrap_or(7);
        let note = s.error.as_deref().unwrap_or_else(|| status_note(s));

        vec![
            Line::from(vec![
                Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(s.project.path.display().to_string()),
            ]),
            Line::from(vec![
                Span::styled("Branch: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(s.branch.label()),
                Span::styled("  Sync: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(sync_label(s)),
                Span::styled(
                    "  Stale Threshold: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{threshold} days")),
                Span::styled(
                    "  Status Note: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(note),
            ]),
            Line::from(vec![
                Span::styled(
                    "Last Commit: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(commit_str),
            ]),
        ]
    };

    let details_block = Paragraph::new(content).block(
        Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(details_block, area);
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
    if !status.has_upstream {
        return "no upstream".to_string();
    }
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let footer_text = if app.is_searching {
        format!(
            "Filter: \"{}\"  [Enter apply / Esc cancel]",
            app.search_query
        )
    } else if !app.search_query.is_empty() {
        format!(
            "Filter: \"{}\"  [Esc clear]  ↑↓/jk nav  enter cd+quit  s sort  d details  r refresh  q quit",
            app.search_query
        )
    } else {
        "↑↓/jk nav  / filter  enter cd+quit  s sort  d details  r refresh  q quit".to_string()
    };

    let style = if app.is_searching {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let footer = Paragraph::new(footer_text).style(style);
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
