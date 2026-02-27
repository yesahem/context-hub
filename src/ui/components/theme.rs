use ratatui::style::{Color, Style};

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(26, 27, 38),           // #1a1b26
            fg: Color::Rgb(192, 202, 245),        // #c0caf5
            primary: Color::Rgb(122, 162, 247),   // #7aa2f7
            secondary: Color::Rgb(187, 154, 247), // #bb9af7
            accent: Color::Rgb(158, 206, 106),    // #9ece6a
            warning: Color::Rgb(224, 175, 104),   // #e0af68
            error: Color::Rgb(247, 118, 142),     // #f7768e
            muted: Color::Rgb(86, 95, 137),       // #565f89
        }
    }

    pub fn default_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary).bg(self.bg)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent).bg(self.bg)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).bg(self.bg)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning).bg(self.bg)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted).bg(self.bg)
    }
}
