use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme_name: &str) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let fallback = "base16-ocean.dark";
        let theme = theme_set
            .themes
            .get(theme_name)
            .cloned()
            .or_else(|| theme_set.themes.get(fallback).cloned())
            .unwrap_or_else(|| theme_set.themes.values().next().cloned().unwrap());

        Self {
            syntax_set,
            theme_set,
            theme,
        }
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        if let Some(t) = self.theme_set.themes.get(theme_name) {
            self.theme = t.clone();
        }
    }

    pub fn find_syntax_for_filename<'a>(&'a self, filename: &str) -> &'a SyntaxReference {
        let lower = filename.to_lowercase();
        if lower.ends_with(".md") || lower.ends_with(".markdown") {
            if let Some(md) = self.syntax_set.find_syntax_by_name("Markdown") {
                return md;
            }
        }
        self
            .syntax_set
            .find_syntax_for_file(filename)
            .ok()
            .and_then(|o| o)
            .unwrap_or_else(|| {
                let ext = filename.rsplit('.').next().unwrap_or("");
                self.syntax_set
                    .find_syntax_by_extension(ext)
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            })
    }

    pub fn highlight_lines_to_ratatui(
        &self,
        lines: &[String],
        syntax: &SyntaxReference,
    ) -> Vec<Line<'static>> {
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        lines
            .iter()
            .map(|line| {
                let line_no_nl = line.trim_end_matches('\n');
                let regions = highlighter
                    .highlight_line(line_no_nl, &self.syntax_set)
                    .unwrap_or_else(|_| vec![(SynStyle::default(), line_no_nl)]);

                let spans: Vec<Span> = regions
                    .into_iter()
                    .map(|(style, text)| Span::styled(text.to_string(), syn_style_to_ratatui(style)))
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

fn syn_style_to_ratatui(style: SynStyle) -> Style {
    let fg = style.foreground;
    let bg = style.background;
    let mut s = Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b));
    if !(bg.r == 0 && bg.g == 0 && bg.b == 0) {
        s = s.bg(Color::Rgb(bg.r, bg.g, bg.b));
    }
    s
}
