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
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::Quit),
            KeyCode::Enter => return Ok(Action::QuitWithSelectedPath),
            KeyCode::Char('r') => app.refresh(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
            _ => {}
        }
    }

    Ok(Action::Continue)
}
