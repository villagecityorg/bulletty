/// Charm-inspired TUI styling extensions.
///
/// Adds gradient backgrounds, category accent colors, styled containers,
/// loading spinners, and form input polish on top of the base theme.
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding},
};

use crate::core::library::settings::theme::Theme;

/// Category accent colors — each category gets a distinct hue from the theme.
/// Maps category name to a theme index (0x8–0xF range).
pub fn category_accent(category: &str, theme: &Theme) -> Color {
    let idx = match category {
        s if s.contains("Science") => 0x8,
        s if s.contains("Space") => 0x9,
        s if s.contains("Nature") => 0xA,
        s if s.contains("History") => 0xB,
        s if s.contains("World") => 0xC,
        s if s.contains("Technology") | s.contains("Tech") => 0xD,
        s if s.contains("Arts") => 0xE,
        _ => 0xF,
    };
    Color::from_u32(theme.base[idx])
}

/// Build a category badge: ` Science ` with accent color on dark background.
pub fn category_badge(category: &str, theme: &Theme) -> Line<'static> {
    let accent = category_accent(category, theme);
    let text = format!(" {} ", category);
    Line::from(Span::styled(
        text,
        Style::new()
            .fg(accent)
            .add_modifier(Modifier::BOLD),
    ))
}

/// A styled container block with rounded borders.
pub fn styled_block<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    let fg = Color::from_u32(theme.base[0x5]);
    Block::bordered()
        .title(title)
        .border_style(Style::new().fg(fg))
        .padding(Padding::new(1, 1, 0, 0))
}

/// A container with an accent-colored border — good for selected/highlighted items.
pub fn accent_block<'a>(category: &str, theme: &Theme) -> Block<'a> {
    let accent = category_accent(category, theme);
    Block::bordered()
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::new().fg(accent))
}

/// Gradient helper: return a Style that interpolates `fg` toward `bg` by `t` (0.0–1.0).
pub fn gradient_style(fg: u32, bg: u32, t: f32) -> Style {
    let r = lerp(((fg >> 16) & 0xFF) as f32, ((bg >> 16) & 0xFF) as f32, t) as u8;
    let g = lerp(((fg >> 8) & 0xFF) as f32, ((bg >> 8) & 0xFF) as f32, t) as u8;
    let b = lerp((fg & 0xFF) as f32, (bg & 0xFF) as f32, t) as u8;
    Style::new().fg(Color::Rgb(r, g, b))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Loading spinner frames.
pub struct Spinner {
    frames: &'static [&'static str],
    tick: usize,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            frames: &["▘", "▝", "▗", "▖"],
            tick: 0,
        }
    }
}

impl Spinner {
    pub fn advance(&mut self) {
        self.tick = (self.tick + 1) % self.frames.len();
    }

    pub fn render(&self, theme: &Theme) -> Span<'static> {
        let fg = Color::from_u32(theme.base[0x8]);
        Span::styled(self.frames[self.tick], Style::new().fg(fg))
    }
}

/// Progress bar using block characters.
pub fn progress_bar(fraction: f32, width: u16, theme: &Theme) -> Line<'static> {
    let n = (fraction * width as f32) as usize;
    let filled = "█".repeat(n);
    let empty = "░".repeat((width as usize).saturating_sub(n));
    let fg = Color::from_u32(theme.base[0x9]);
    Line::from(Span::styled(format!("{}{}", filled, empty), Style::new().fg(fg)))
}

/// Styled text input — a bordered input box with the given label.
pub fn input_block<'a>(label: &'a str, focused: bool, theme: &Theme) -> Block<'a> {
    let fg = if focused {
        Color::from_u32(theme.base[0xD])
    } else {
        Color::from_u32(theme.base[0x6])
    };
    Block::bordered()
        .title(label)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::new().fg(fg))
}
