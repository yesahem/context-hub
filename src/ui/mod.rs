pub mod components;
pub mod screens;

use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;

use screens::context::ContextScreen;
use screens::sync::SyncScreen;

pub enum AppState {
    Sync(SyncScreen),
    Context(ContextScreen),
    Exit,
}

pub struct App {
    pub state: AppState,
    pub should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Exit,
            should_exit: false,
        }
    }

    pub fn run_sync(commits: Vec<crate::core::git::CommitInfo>) -> io::Result<()> {
        use crossterm::event::{read, Event, KeyCode};

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        let mut screen = SyncScreen::new(commits);

        loop {
            terminal.draw(|f: &mut Frame<'_>| {
                screen.render(f);
            })?;

            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Up => screen.move_up(),
                    KeyCode::Down => screen.move_down(),
                    KeyCode::Char(' ') => screen.toggle_selection(),
                    KeyCode::Enter => {
                        screen.status = screens::sync::SyncStatus::Processing;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn run_context(contexts: Vec<crate::core::storage::GlobalContext>) -> io::Result<()> {
        use crossterm::event::{read, Event, KeyCode};

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        let mut screen = ContextScreen::new(contexts);

        loop {
            terminal.draw(|f: &mut Frame<'_>| {
                screen.render(f);
            })?;

            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Up => screen.move_up(),
                    KeyCode::Down => screen.move_down(),
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
