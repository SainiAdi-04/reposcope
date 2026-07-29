use std::io;

use crossterm::event::{self, Event, KeyCode};

use super::App;

pub enum Action {
    Continue,
    Quit,
    QuitWithSelectedPath,
}

pub fn handle_event(app: &mut App) -> io::Result<Action> {
    if let Event::Key(key) = event::read()? {
        if app.is_searching {
            match key.code {
                KeyCode::Esc => {
                    app.is_searching = false;
                    app.search_query.clear();
                    app.clamp_selection();
                }
                KeyCode::Enter => {
                    app.is_searching = false;
                    app.clamp_selection();
                }
                KeyCode::Backspace => {
                    app.search_query.pop();
                    app.clamp_selection();
                }
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                    app.clamp_selection();
                }
                _ => {}
            }
            return Ok(Action::Continue);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(Action::Quit),
            KeyCode::Esc => {
                if !app.search_query.is_empty() {
                    app.search_query.clear();
                    app.clamp_selection();
                } else {
                    return Ok(Action::Quit);
                }
            }
            KeyCode::Enter => return Ok(Action::QuitWithSelectedPath),
            KeyCode::Char('/') => {
                app.is_searching = true;
            }
            KeyCode::Char('s') => app.cycle_sort(),
            KeyCode::Char('d') => app.toggle_details(),
            KeyCode::Char('r') => app.refresh(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
            KeyCode::Home | KeyCode::Char('g') => app.select_first(),
            KeyCode::End | KeyCode::Char('G') => app.select_last(),
            KeyCode::PageDown => app.select_page_down(10),
            KeyCode::PageUp => app.select_page_up(10),
            _ => {}
        }
    }

    Ok(Action::Continue)
}
