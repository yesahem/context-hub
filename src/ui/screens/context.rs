use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

use crate::core::storage::GlobalContext;
use crate::ui::components::theme::Theme;

pub struct ContextScreen {
    pub contexts: Vec<GlobalContext>,
    pub scroll: u16,
    pub current_index: usize,
}

impl ContextScreen {
    pub fn new(contexts: Vec<GlobalContext>) -> Self {
        Self {
            contexts,
            scroll: 0,
            current_index: 0,
        }
    }

    pub fn render(&self, f: &mut Frame<'_>) {
        let theme = Theme::tokyo_night();
        let size = f.area();

        use ratatui::widgets::Borders;

        if self.contexts.is_empty() {
            let empty = Paragraph::new(
                "No context stored.\nRun 'ctxhub sync' to extract context from commits.",
            )
            .style(theme.muted_style())
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(empty, size);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let title = Paragraph::new("Repository Context")
            .style(theme.primary_style())
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = self
            .contexts
            .iter()
            .enumerate()
            .map(|(_i, c)| {
                let msg = c.commit_message.lines().next().unwrap_or("No message");
                ListItem::new(format!(
                    "{} - {}",
                    &c.commit_hash[..7.min(c.commit_hash.len())],
                    msg
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Commits").borders(Borders::ALL))
            .style(theme.default_style());

        f.render_widget(list, chunks[1]);

        let hint = Paragraph::new("Press ESC to exit")
            .style(theme.muted_style())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(hint, chunks[2]);
    }

    pub fn move_up(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.current_index < self.contexts.len() - 1 {
            self.current_index += 1;
        }
    }
}
