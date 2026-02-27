use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

pub struct Logo;

impl Widget for Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logo_text = r#"
   ___      _ _           _   
  / __\_ __(_) |_ _ __ __| |  
 _\ \ / '__| | __| '__/ _` |  
/ /__\ |  | | |_| | | (_| |  
\____/_|  |_|\__|_|  \__,_|  
                              
   C O N T E X T   H U B      
"#;

        let theme = super::theme::Theme::tokyo_night();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.primary_style())
            .border_type(ratatui::widgets::BorderType::Rounded);

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, line) in logo_text.lines().enumerate() {
            let x = inner.x + (inner.width.saturating_sub(line.len() as u16) / 2);
            let y = inner.y + 2 + i as u16;
            if y < inner.y + inner.height {
                buf.set_string(x, y, line, theme.primary_style());
            }
        }
    }
}

pub struct ProgressBar {
    pub current: usize,
    pub total: usize,
    pub label: String,
}

impl ProgressBar {
    pub fn new(current: usize, total: usize, label: &str) -> Self {
        Self {
            current,
            total,
            label: label.to_string(),
        }
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = super::theme::Theme::tokyo_night();

        let block = Block::default()
            .title(self.label.as_str())
            .borders(Borders::ALL)
            .border_style(theme.primary_style());

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width > 2 && inner.height > 2 {
            let bar_width = inner.width - 2;
            let progress = if self.total > 0 {
                (self.current as f64 / self.total as f64 * bar_width as f64) as u16
            } else {
                0
            };

            for x in 0..bar_width {
                let char = if x < progress { '█' } else { '░' };
                buf.set_string(
                    inner.x + 1 + x,
                    inner.y + 1,
                    char.to_string(),
                    theme.accent_style(),
                );
            }
        }
    }
}

pub struct SelectionList {
    pub items: Vec<String>,
    pub selected: Vec<bool>,
    pub scroll: u16,
}

impl SelectionList {
    pub fn new(items: Vec<String>) -> Self {
        let selected = vec![false; items.len()];
        Self {
            items,
            selected,
            scroll: 0,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if index < self.selected.len() {
            self.selected[index] = !self.selected[index];
        }
    }

    pub fn select(&mut self, index: usize) {
        self.selected = vec![false; self.selected.len()];
        if index < self.selected.len() {
            self.selected[index] = true;
        }
    }
}

impl Widget for SelectionList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = super::theme::Theme::tokyo_night();

        let block = Block::default()
            .title("Select Commits")
            .borders(Borders::ALL)
            .border_style(theme.primary_style());

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, item) in self.items.iter().enumerate() {
            let y = inner.y + i as u16;
            if y < inner.y + inner.height && i >= self.scroll as usize {
                let prefix = if self.selected[i] { "◉" } else { "○" };
                let style = if self.selected[i] {
                    theme.accent_style()
                } else {
                    theme.default_style()
                };
                buf.set_string(inner.x + 1, y, format!("{} {}", prefix, item), style);
            }
        }
    }
}
