use ratatui::style::Color;
use ratatui::text::{Line, Span};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SyntectColor, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// Global singleton for syntax set and theme set to avoid loading on every render
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

pub fn init() {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    THEME_SET.get_or_init(ThemeSet::load_defaults);
}

pub fn highlight<'a>(text: &'a str, extension: &str) -> Vec<Line<'a>> {
    let ps = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);

    // Find syntax
    let syntax = ps
        .find_syntax_by_extension(extension)
        .or_else(|| ps.find_syntax_by_extension("txt"))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    // Use a theme - "base16-ocean.dark" is usually good for TUI
    // Use a theme - try "base16-ocean.dark", fallback to first available
    let theme_name = "base16-ocean.dark";
    let theme = ts.themes.get(theme_name)
        .or_else(|| ts.themes.values().next())
        .unwrap_or_else(|| panic!("No themes available in syntect"));

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(text) {
        let ranges: Vec<(syntect::highlighting::Style, &str)> =
            h.highlight_line(line, ps).unwrap_or_default();
        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(style, content)| {
                let fg_color = to_ratatui_color(style.foreground);
                // We generally ignore background to blend with TUI, or we could support it
                Span::styled(
                    content.to_string(), // We have to own the string here because Span life-times are tricky with syntect yields
                    ratatui::style::Style::default().fg(fg_color),
                )
            })
            .collect();
        lines.push(Line::from(spans));
    }

    lines
}

fn to_ratatui_color(c: SyntectColor) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}
