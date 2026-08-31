use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn bg() -> Color { Color::Rgb(15, 17, 23) }           // #0f1117
    pub fn card() -> Color { Color::Rgb(26, 29, 39) }         // #1a1d27
    pub fn card_selected() -> Color { Color::Rgb(34, 38, 52) }
    pub fn border() -> Color { Color::Rgb(42, 46, 63) }       // #2a2e3f
    pub fn border_accent() -> Color { Color::Rgb(124, 140, 255) } // #7c8cff
    pub fn text() -> Color { Color::Rgb(230, 233, 245) }
    pub fn muted() -> Color { Color::Rgb(139, 143, 163) }     // #8b8fa3
    pub fn dim() -> Color { Color::Rgb(90, 95, 115) }
    pub fn accent() -> Color { Color::Rgb(124, 140, 255) }
    pub fn accent_dim() -> Color { Color::Rgb(90, 105, 200) }
    pub fn success() -> Color { Color::Rgb(61, 214, 140) }    // #3dd68c
    pub fn warning() -> Color { Color::Rgb(245, 165, 36) }
    pub fn error() -> Color { Color::Rgb(255, 107, 107) }

    pub fn style_text() -> Style { Style::default().fg(Self::text()) }
    pub fn style_muted() -> Style { Style::default().fg(Self::muted()) }
    pub fn style_accent() -> Style { Style::default().fg(Self::accent()).add_modifier(Modifier::BOLD) }
    pub fn style_success() -> Style { Style::default().fg(Self::success()).add_modifier(Modifier::BOLD) }
    pub fn style_border() -> Style { Style::default().fg(Self::border()) }
    pub fn style_border_accent() -> Style { Style::default().fg(Self::border_accent()) }
}
