//! Семантические стили над палитрой. Виджеты обращаются к ролям, а не к
//! буквальным цветам — см. principles.md, раздел «Цвет и темы».

use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

impl Palette {
    pub const DARK: Palette = Palette {
        background: Color::Black,
        foreground: Color::Gray,
        accent: Color::Cyan,
        muted: Color::DarkGray,
        success: Color::Green,
        warning: Color::Yellow,
        error: Color::Red,
        selection_bg: Color::Rgb(45, 45, 85),
        selection_fg: Color::Cyan,
    };

    #[allow(dead_code)]
    pub const LIGHT: Palette = Palette {
        background: Color::White,
        foreground: Color::Rgb(40, 40, 40),
        accent: Color::Blue,
        muted: Color::Rgb(120, 120, 120),
        success: Color::Rgb(0, 120, 0),
        warning: Color::Rgb(160, 120, 0),
        error: Color::Rgb(180, 0, 0),
        selection_bg: Color::Rgb(220, 220, 255),
        selection_fg: Color::Rgb(0, 0, 120),
    };
}

pub struct Theme(Palette);

impl Theme {
    pub fn new(palette: Palette) -> Self {
        Theme(palette)
    }

    pub fn accent(&self) -> Style {
        Style::default()
            .fg(self.0.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal(&self) -> Style {
        Style::default().fg(self.0.foreground)
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.0.muted)
    }

    pub fn success(&self) -> Style {
        Style::default()
            .fg(self.0.success)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning(&self) -> Style {
        Style::default()
            .fg(self.0.warning)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error(&self) -> Style {
        Style::default()
            .fg(self.0.error)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.0.muted)
    }

    pub fn selected(&self) -> Style {
        Style::default()
            .bg(self.0.selection_bg)
            .fg(self.0.selection_fg)
    }

    pub fn background(&self) -> Style {
        Style::default().bg(self.0.background)
    }

    pub fn status_bar(&self) -> Style {
        Style::default()
            .fg(self.0.foreground)
            .bg(self.0.selection_bg)
    }

    /// Цветная «пилюля» состояния: цветной фон + цвет фона как текст.
    pub fn badge(&self, color: Color) -> Style {
        Style::default()
            .fg(self.0.background)
            .bg(color)
            .add_modifier(Modifier::BOLD)
    }
}
