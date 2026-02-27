use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::ui::components::theme::Theme;

pub struct InitScreen {
    pub step: InitStep,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitStep {
    Welcome,
    Creating,
    Success,
    Error,
}

impl InitScreen {
    pub fn new() -> Self {
        Self {
            step: InitStep::Welcome,
            message: String::new(),
        }
    }

    pub fn render(&self, f: &mut Frame<'_>) {
        let theme = Theme::tokyo_night();
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(size);

        let logo = r#"
   ___      _ _           _   
  / __\_ __(_) |_ _ __ __| |  
 _\ \ / '__| | __| '__/ _` |  
/ /__\ |  | | |_| | | (_| |  
\____/_|  |_|\__|_|  \__,_|  
                              
   C O N T E X T   H U B      
"#;

        let title = Paragraph::new(logo)
            .style(theme.primary_style())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(title, chunks[0]);

        let content = match self.step {
            InitStep::Welcome => "Press ENTER to initialize ContextHub in this directory",
            InitStep::Creating => "Creating .contexthub/ directory...",
            InitStep::Success => "ContextHub initialized successfully!",
            InitStep::Error => &self.message,
        };

        use ratatui::widgets::BorderType;

        let message_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.primary_style())
            .title("Status");

        let message = Paragraph::new(content)
            .style(theme.default_style())
            .alignment(ratatui::layout::Alignment::Center)
            .block(message_block);

        f.render_widget(message, chunks[1]);

        let hint = Paragraph::new("Press ESC to exit")
            .style(theme.muted_style())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(hint, chunks[3]);
    }

    pub fn next_step(&mut self) {
        match self.step {
            InitStep::Welcome => self.step = InitStep::Creating,
            InitStep::Creating => self.step = InitStep::Success,
            InitStep::Success => self.step = InitStep::Success,
            InitStep::Error => self.step = InitStep::Error,
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.step = InitStep::Error;
        self.message = msg;
    }
}
