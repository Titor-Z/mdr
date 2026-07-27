use ratatui::text::Line;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub text: String,
    pub line: usize,
}

pub(crate) struct RenderCtx<'a> {
    pub lines: Vec<Line<'static>>,
    pub list_indent: Vec<usize>,
    pub syn_set: &'a SyntaxSet,
    pub theme_set: &'a ThemeSet,
    pub viewport_width: usize,
}
