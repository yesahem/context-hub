use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::core::git::CommitInfo;
use crate::ui::components::theme::Theme;

pub struct SyncScreen {
    pub commits: Vec<CommitInfo>,
    pub selected_indices: Vec<usize>,
    pub current_index: usize,
    pub scroll: u16,
    pub status: SyncStatus,
    pub processing_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncStatus {
    Selection,
    Processing,
    Complete,
    Error,
}

impl SyncScreen {
    pub fn new(commits: Vec<CommitInfo>) -> Self {
        Self {
            commits,
            selected_indices: vec![0],
            current_index: 0,
            scroll: 0,
            status: SyncStatus::Selection,
            processing_index: 0,
        }
    }

    pub fn render(&self, f: &mut Frame<'_>) {
        let theme = Theme::tokyo_night();
        let size = f.area();

        match self.status {
            SyncStatus::Selection => self.render_selection(f, size, &theme),
            SyncStatus::Processing => self.render_processing(f, size, &theme),
            SyncStatus::Complete => self.render_complete(f, size, &theme),
            SyncStatus::Error => self.render_error(f, size, &theme),
        }
    }

    fn render_selection(&self, f: &mut Frame<'_>, size: ratatui::layout::Rect, theme: &Theme) {
        use ratatui::widgets::Borders;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let title = Paragraph::new("Select commits to process (SPACE to toggle, ENTER to proceed)")
            .style(theme.primary_style())
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = self
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let prefix = if self.selected_indices.contains(&i) {
                    "◉"
                } else {
                    "○"
                };
                let line = format!(
                    "{} - {}",
                    c.short_hash,
                    c.message.lines().next().unwrap_or("")
                );
                ListItem::new(format!("{} {}", prefix, line))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Commits")
                    .borders(Borders::ALL)
                    .border_style(theme.primary_style()),
            )
            .style(theme.default_style());

        f.render_widget(list, chunks[1]);

        let hint = Paragraph::new("SPACE Select  ENTER Process  ESC Cancel")
            .style(theme.muted_style())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(hint, chunks[2]);
    }

    fn render_processing(&self, f: &mut Frame<'_>, size: ratatui::layout::Rect, theme: &Theme) {
        use ratatui::widgets::Borders;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(5),
            ])
            .split(size);

        let title = Paragraph::new("Processing commits...")
            .style(theme.primary_style())
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let progress = if self.commits.is_empty() {
            0.0
        } else {
            self.processing_index as f64 / self.commits.len() as f64
        };

        let progress_bar = Gauge::default()
            .ratio(progress)
            .label(format!(
                "{}/{}",
                self.processing_index + 1,
                self.commits.len()
            ))
            .style(theme.accent_style())
            .block(Block::default().title("Progress").borders(Borders::ALL));

        f.render_widget(progress_bar, chunks[2]);

        if self.processing_index < self.commits.len() {
            let commit = &self.commits[self.processing_index];
            let info = Paragraph::new(format!(
                "Processing: {} - {}",
                commit.short_hash,
                commit.message.lines().next().unwrap_or("")
            ))
            .style(theme.default_style());
            f.render_widget(info, chunks[1]);
        }
    }

    fn render_complete(&self, f: &mut Frame<'_>, size: ratatui::layout::Rect, theme: &Theme) {
        use ratatui::widgets::Borders;

        let msg = format!("Processed {} commits successfully!", self.commits.len());
        let paragraph = Paragraph::new(msg)
            .style(theme.accent_style())
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.accent_style()),
            );
        f.render_widget(paragraph, size);
    }

    fn render_error(&self, f: &mut Frame<'_>, size: ratatui::layout::Rect, theme: &Theme) {
        use ratatui::widgets::Borders;

        let paragraph = Paragraph::new("Error processing commits.")
            .style(theme.error_style())
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.error_style()),
            );
        f.render_widget(paragraph, size);
    }

    pub fn move_up(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            if self.current_index < self.scroll as usize {
                self.scroll = self.current_index as u16;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.current_index < self.commits.len() - 1 {
            self.current_index += 1;
            if self.current_index >= self.scroll as usize + 10 {
                self.scroll += 1;
            }
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(pos) = self
            .selected_indices
            .iter()
            .position(|&i| i == self.current_index)
        {
            self.selected_indices.remove(pos);
        } else {
            self.selected_indices.push(self.current_index);
        }
    }

    pub fn get_selected_commits(&self) -> Vec<CommitInfo> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.commits.get(i).cloned())
            .collect()
    }
}
