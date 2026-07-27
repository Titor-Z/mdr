use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::FontStyle as SyntectFontStyle;

pub fn heading_style(level: u8) -> Style {
    let color = match level {
        1 => Color::Rgb(255, 200, 0),
        2 => Color::Rgb(255, 215, 0),
        3 => Color::Rgb(0, 200, 255),
        4 => Color::Rgb(0, 255, 128),
        5 => Color::Rgb(180, 180, 180),
        _ => Color::Rgb(128, 128, 128),
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub fn syntect_fg_to_ratatui(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

pub fn syntect_bg_to_ratatui(color: syntect::highlighting::Color) -> Option<Color> {
    if color.a == 0 { None } else { Some(Color::Rgb(color.r, color.g, color.b)) }
}

pub fn syntect_font_style_to_ratatui(font_style: SyntectFontStyle) -> Modifier {
    let mut m = Modifier::empty();
    if font_style.contains(SyntectFontStyle::BOLD) {
        m |= Modifier::BOLD;
    }
    if font_style.contains(SyntectFontStyle::ITALIC) {
        m |= Modifier::ITALIC;
    }
    if font_style.contains(SyntectFontStyle::UNDERLINE) {
        m |= Modifier::UNDERLINED;
    }
    m
}
