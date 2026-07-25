use anyhow::Result;
use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;
use syntect::highlighting::{FontStyle as SyntectFontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

// Comrak 0.33 uses Arena with simpler lifetime; re-export for convenience.
type AstNodeRef<'a> = &'a AstNode<'a>;

/// Information about a link in the rendered output.
#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub text: String,
    /// 0-based line index in the rendered output.
    pub line: usize,
}

/// Render markdown content into a vector of ratatui Lines for display.
pub fn render(content: &str) -> Result<Vec<Line<'static>>> {
    render_with_width(content, 80)
}

/// Render with a known viewport width (used for table column sizing).
/// Internally reserves 1 column for the TUI scrollbar.
pub fn render_with_width(content: &str, viewport_width: usize) -> Result<Vec<Line<'static>>> {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    let root = parse_document(&arena, content, &options);

    let syn_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    // Reserve 1 column for left padding and 1 for right padding/scrollbar
    let content_width = viewport_width.saturating_sub(2).max(20);

    let mut ctx = RenderCtx {
        lines: Vec::new(),
        list_indent: Vec::new(),
        syn_set: &syn_set,
        theme_set: &theme_set,
        viewport_width: content_width,
    };

    render_node(root, &mut ctx, 0);
    Ok(ctx.lines)
}

/// Extract links from rendered lines.
/// Our renderer formats links as `text [url]`, so we scan for `[url]` patterns.
pub fn extract_links(lines: &[Line<'static>]) -> Vec<LinkInfo> {
    let mut links = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let text = line.to_string();
        // Look for [url] at the end of the line (after link text)
        if let Some(open) = text.rfind('[') {
            if let Some(close) = text[open..].find(']') {
                let url = &text[open + 1..open + close];
                if url.starts_with("http://") || url.starts_with("https://") {
                    let link_text = text[..open].trim().to_string();
                    links.push(LinkInfo {
                        url: url.to_string(),
                        text: link_text,
                        line: i,
                    });
                }
            }
        }
    }
    links
}

struct RenderCtx<'a> {
    lines: Vec<Line<'static>>,
    /// Stack of list indentation levels.
    list_indent: Vec<usize>,
    syn_set: &'a SyntaxSet,
    theme_set: &'a ThemeSet,
    /// Viewport width for table layout (default 80).
    viewport_width: usize,
}

// ── block-level rendering ────────────────────────────────────────────

fn render_node<'a>(node: AstNodeRef<'a>, ctx: &mut RenderCtx<'a>, depth: usize) {
    let data = node.data.borrow();

    match &data.value {
        NodeValue::Document => {
            for child in node.children() {
                render_node(child, ctx, depth);
            }
        }

        NodeValue::Heading(h) => {
            let style = heading_style(h.level);
            let mut spans = vec![];

            // Collect inline content
            for child in node.children() {
                collect_inline(child, ctx, &mut spans);
            }

            // Add a marker for heading level
            let prefix = "#".repeat(h.level as usize);
            let marker = Span::styled(format!("{} ", prefix), style);
            spans.insert(0, marker);

            // Blank line before heading (except at top)
            if !ctx.lines.is_empty() {
                ctx.lines.push(Line::from(""));
            }
            // If heading has inline children, pack them into one line
            if !spans.is_empty() {
                ctx.lines.push(Line::from(spans));
            }
            ctx.lines.push(Line::from(""));
        }

        NodeValue::Paragraph => {
            let mut spans = vec![];
            for child in node.children() {
                collect_inline(child, ctx, &mut spans);
            }
            if !spans.is_empty() {
                let indent = ctx.list_indent.last().copied().unwrap_or(0);
                let max_w = ctx.viewport_width.saturating_sub(indent).max(20);
                let wrapped = wrap_spans(&spans, max_w);
                for line_spans in &wrapped {
                    let mut full = vec![Span::raw(" ".repeat(indent))];
                    full.extend(line_spans.iter().cloned());
                    ctx.lines.push(Line::from(full));
                }
                ctx.lines.push(Line::from(""));
            }
        }

        NodeValue::CodeBlock(cb) => {
            let lang = cb.info.trim();
            let code = &cb.literal;
            let theme = &ctx.theme_set.themes["base16-ocean.dark"];

            let mut highlighter = if lang.is_empty() {
                None
            } else {
                ctx.syn_set
                    .find_syntax_by_token(lang)
                    .map(|s| HighlightLines::new(s, theme))
            };

            ctx.lines.push(Line::from("")); // blank line before

            for line in code.lines() {
                let spans = if let Some(ref mut hl) = highlighter {
                    let ranges = hl.highlight_line(line, ctx.syn_set).unwrap();
                    ranges
                        .into_iter()
                        .map(|(style, text)| {
                            let fg = syntect_fg_to_ratatui(style.foreground);
                            let bg = syntect_bg_to_ratatui(style.background);
                            let modifier = syntect_font_style_to_ratatui(style.font_style);
                            Span::styled(
                                text.to_string(),
                                Style::default()
                                    .fg(fg)
                                    .bg(bg.unwrap_or(Color::Rgb(43, 48, 59))) // dark bg
                                    .add_modifier(modifier),
                            )
                        })
                        .collect()
                } else {
                    // Plain text with code block background
                    vec![Span::styled(
                        line.to_string(),
                        Style::default().bg(Color::Rgb(43, 48, 59)),
                    )]
                };
                ctx.lines.push(Line::from(spans));
            }

            ctx.lines.push(Line::from("")); // blank line after
        }

        NodeValue::List(list) => {
            let indent = ctx.list_indent.last().copied().unwrap_or(0) + 2;
            ctx.list_indent.push(indent);
            let mut idx = 0usize;
            for child in node.children() {
                if matches!(&child.data.borrow().value, NodeValue::Item(_) | NodeValue::TaskItem(_)) {
                    render_list_item(child, ctx, list.list_type, &mut idx, indent);
                }
            }
            ctx.list_indent.pop();
        }

        NodeValue::BlockQuote => {
            for child in node.children() {
                let start_line = ctx.lines.len();
                render_node(child, ctx, depth + 1);
                // Prepend quote marker to lines added by children
                for line in ctx.lines.iter_mut().skip(start_line) {
                    if !line.spans.is_empty() {
                        let marker = Span::styled(
                            "▌ ",
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
                        );
                        line.spans.insert(0, marker);
                    }
                }
            }
        }

        NodeValue::ThematicBreak => {
            ctx.lines.push(Line::from(Span::styled(
                "─".repeat(80),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
            )));
            ctx.lines.push(Line::from(""));
        }

        NodeValue::Table(_alignments) => {
            // ── 1. collect raw rows/cells ──
            let raw_rows: Vec<Vec<Vec<Span<'static>>>> = node
                .children()
                .filter_map(|row_node| {
                    if matches!(&row_node.data.borrow().value, NodeValue::TableRow(_)) {
                        let cells: Vec<Vec<Span<'static>>> = row_node
                            .children()
                            .filter_map(|cell_node| {
                                if matches!(&cell_node.data.borrow().value, NodeValue::TableCell) {
                                    let mut spans = vec![];
                                    for child in cell_node.children() {
                                        collect_inline(child, ctx, &mut spans);
                                    }
                                    Some(spans)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Some(cells)
                    } else {
                        None
                    }
                })
                .collect();

            if raw_rows.is_empty() {
                return;
            }

            let num_cols = raw_rows.iter().map(|r| r.len()).max().unwrap_or(0);
            if num_cols == 0 {
                return;
            }

            // ── 2. natural column widths ──
            let mut natural = vec![0usize; num_cols];
            for row in &raw_rows {
                for (ci, cell) in row.iter().enumerate() {
                    let w: usize = cell.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
                    natural[ci] = natural[ci].max(w);
                }
            }

            // ── 3. distribute viewport width (content area only) ──
            let indent = ctx.list_indent.last().copied().unwrap_or(0);
            // border: 2 (left+right) + (num_cols-1) vertical separators
            let border_chars = 2 + num_cols.saturating_sub(1);
            // cell padding: 2 per column (space on each side of content)
            let padding_total = num_cols * 2;
            let available = ctx
                .viewport_width
                .saturating_sub(indent + border_chars + padding_total);
            let natural_total: usize = natural.iter().sum();

            let col_widths: Vec<usize> = if natural_total == 0 {
                vec![4; num_cols]
            } else if natural_total <= available {
                let extra = available - natural_total;
                let mut widths = natural.clone();
                let mut remaining = extra;
                while remaining > 0 {
                    for w in widths.iter_mut() {
                        if remaining == 0 {
                            break;
                        }
                        *w += 1;
                        remaining -= 1;
                    }
                }
                widths
            } else {
                let ratio = available as f64 / natural_total as f64;
                natural
                    .iter()
                    .map(|&n| ((n as f64 * ratio).round() as usize).max(4))
                    .collect()
            };

            // ── 4. wrap each cell ──
            let mut wrapped_rows: Vec<Vec<Vec<Vec<Span<'static>>>>> = Vec::new();
            for row in &raw_rows {
                let mut wrapped_cells = Vec::new();
                for (ci, cell) in row.iter().enumerate() {
                    let cw = col_widths.get(ci).copied().unwrap_or(10);
                    wrapped_cells.push(wrap_spans(cell, cw));
                }
                wrapped_rows.push(wrapped_cells);
            }

            // ── 5. vertically centre each row ──
            let border = Style::default().fg(Color::Rgb(100, 100, 110));
            let header_style = Style::default().add_modifier(Modifier::BOLD);

            // Helper: draw a border line
            let mk_border = |left: &str, _mid: &str, right: &str, sep: &str| -> Line {
                let mut spans: Vec<Span<'static>> = vec![];
                if indent > 0 {
                    spans.push(Span::raw(" ".repeat(indent)));
                }
                spans.push(Span::styled(left.to_string(), border));
                for ci in 0..num_cols {
                    let cw = col_widths[ci];
                    spans.push(Span::styled("─".repeat(cw + 2), border));
                    if ci + 1 < num_cols {
                        spans.push(Span::styled(sep.to_string(), border));
                    } else {
                        spans.push(Span::styled(right.to_string(), border));
                    }
                }
                Line::from(spans)
            };

            ctx.lines.push(Line::from(""));

            // Top border
            ctx.lines.push(mk_border("┌", "┬", "┐", "┬"));

            for (ri, wrapped_cells) in wrapped_rows.iter().enumerate() {
                let max_lines = wrapped_cells.iter().map(|c| c.len()).max().unwrap_or(1).max(1);

                // Content rows (including header)
                for line_i in 0..max_lines {
                    let mut spans: Vec<Span<'static>> = vec![];
                    if indent > 0 {
                        spans.push(Span::raw(" ".repeat(indent)));
                    }
                    spans.push(Span::styled("│", border));

                    for (ci, cell_lines) in wrapped_cells.iter().enumerate() {
                        let cw = col_widths[ci];

                        spans.push(Span::raw(" ")); // left padding

                        if line_i < cell_lines.len() {
                            let line_spans = &cell_lines[line_i];
                            let line_w: usize = line_spans
                                .iter()
                                .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                                .sum();
                            let pad = cw.saturating_sub(line_w);

                            if ri == 0 {
                                for sp in line_spans {
                                    spans.push(Span::styled(
                                        sp.content.clone(),
                                        sp.style.patch(header_style),
                                    ));
                                }
                            } else {
                                spans.extend(line_spans.iter().cloned());
                            }
                            spans.push(Span::raw(" ".repeat(pad + 1))); // right padding
                        } else {
                            spans.push(Span::raw(" ".repeat(cw + 1))); // empty cell
                        }

                        if ci + 1 < num_cols {
                            spans.push(Span::styled("│", border));
                        } else {
                            spans.push(Span::styled("│", border));
                        }
                    }

                    ctx.lines.push(Line::from(spans));
                }

                // Separator between rows
                if ri == 0 && wrapped_rows.len() > 1 {
                    // Header → body separator
                    ctx.lines.push(mk_border("├", "┼", "┤", "┼"));
                } else if ri > 0 && ri + 1 < wrapped_rows.len() {
                    ctx.lines.push(mk_border("├", "┼", "┤", "┼"));
                }
            }

            // Bottom border
            ctx.lines.push(mk_border("└", "┴", "┘", "┴"));
            ctx.lines.push(Line::from(""));
        }

        _ => {
            for child in node.children() {
                render_node(child, ctx, depth);
            }
        }
    }
}

fn render_list_item<'a>(
    node: AstNodeRef<'a>,
    ctx: &mut RenderCtx<'a>,
    list_type: ListType,
    idx: &mut usize,
    indent: usize,
) {
    let bullet = match list_type {
        ListType::Bullet => "•".to_string(),
        ListType::Ordered => {
            *idx += 1;
            format!("{}.", *idx)
        }
    };

    // Check if this is a task list item (the node itself is TaskItem, not Item)
    let is_task = matches!(&node.data.borrow().value, NodeValue::TaskItem(_));
    let checked = matches!(&node.data.borrow().value, NodeValue::TaskItem(Some(_)));

    let (bullet_str, bullet_color) = if is_task {
        if checked {
            ("☑".to_string(), Color::Rgb(80, 200, 120))
        } else {
            ("☐".to_string(), Color::Rgb(180, 180, 180))
        }
    } else {
        (bullet, Color::Cyan)
    };
    let bullet_span = Span::styled(bullet_str, Style::default().fg(bullet_color));

    // Collect children for rendering (skip TaskItem, it's been handled)
    let child_indices: Vec<_> = node.children().collect();
    let mut first = true;

    for child in child_indices {
        let child_data = child.data.borrow();
        match &child_data.value {
            NodeValue::Paragraph => {
                let mut content_spans = vec![];
                for inline_child in child.children() {
                    collect_inline(inline_child, ctx, &mut content_spans);
                }

                if !content_spans.is_empty() {
                    // Calculate available width
                    let bullet_prefix_width = if first {
                        indent.saturating_sub(2) + 2 // spaces + bullet + space
                    } else {
                        indent
                    };
                    let max_w = ctx.viewport_width.saturating_sub(bullet_prefix_width).max(10);
                    let wrapped = wrap_spans(&content_spans, max_w);

                    for (wi, line_spans) in wrapped.iter().enumerate() {
                        let mut full = vec![];
                        if wi == 0 && first {
                            // First line: bullet prefix
                            full.push(Span::raw(" ".repeat(indent.saturating_sub(2))));
                            full.push(bullet_span.clone());
                            full.push(Span::raw(" "));
                        } else {
                            full.push(Span::raw(" ".repeat(indent)));
                        }
                        full.extend(line_spans.iter().cloned());
                        ctx.lines.push(Line::from(full));
                    }
                }
                first = false;
            }
            _ => {
                // Nested list — delegate to render_node which handles
                // its own indentation via list_indent stack.
                render_node(child, ctx, 0);
                first = false;
            }
        }
    }
}

// ── inline rendering ─────────────────────────────────────────────────

fn collect_inline<'a>(node: AstNodeRef<'a>, ctx: &RenderCtx<'a>, spans: &mut Vec<Span<'static>>) {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Text(t) => {
            let text = t.clone();
            spans.push(Span::raw(text));
        }
        NodeValue::SoftBreak => {
            // Treat as space; in non-inline context this creates a new line,
            // but within a paragraph we just add a space.
        }
        NodeValue::LineBreak => {
            // Hard line break → we can just ignore and let text flow,
            // or insert a space. For now, a space.
        }
        NodeValue::Code(c) => {
            let code_text = c.literal.clone();
            spans.push(Span::styled(
                code_text,
                Style::default()
                    .bg(Color::Rgb(60, 60, 60))
                    .fg(Color::Rgb(203, 170, 111)),
            ));
        }
        NodeValue::Emph => {
            let child_spans = collect_inline_children(node, ctx);
            for span in child_spans {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::ITALIC),
                ));
            }
        }
        NodeValue::Strong => {
            let child_spans = collect_inline_children(node, ctx);
            for span in child_spans {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::BOLD),
                ));
            }
        }
        NodeValue::Strikethrough => {
            let child_spans = collect_inline_children(node, ctx);
            for span in child_spans {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::CROSSED_OUT),
                ));
            }
        }
        NodeValue::Link(link) => {
            let mut child_spans = collect_inline_children(node, ctx);
            // Append URL after text in dim style
            for span in &mut child_spans {
                let new_style = span.style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED);
                span.style = new_style;
            }
            // Also show URL dimly after the linked text
            if !link.url.is_empty() {
                child_spans.push(Span::styled(
                    format!(" [{}]", link.url),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ));
            }
            spans.extend(child_spans);
        }
        NodeValue::Image(img) => {
            // For images, show alt text and URL
            let alt_text = img.title.clone();
            let url = &img.url;
            let display = if !alt_text.is_empty() {
                format!("[image: {}]({})", alt_text, url)
            } else {
                format!("[image]({})", url)
            };
            spans.push(Span::styled(
                display,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            ));
        }
        _ => {
            for child in node.children() {
                collect_inline(child, ctx, spans);
            }
        }
    }
}

fn collect_inline_children<'a>(node: AstNodeRef<'a>, ctx: &RenderCtx<'a>) -> Vec<Span<'static>> {
    let mut spans = vec![];
    for child in node.children() {
        collect_inline(child, ctx, &mut spans);
    }
    spans
}

// ── style helpers ────────────────────────────────────────────────────

fn heading_style(level: u8) -> Style {
    let color = match level {
        1 => Color::Rgb(255, 200, 0),   // bright gold
        2 => Color::Rgb(255, 215, 0),   // gold
        3 => Color::Rgb(0, 200, 255),   // cyan
        4 => Color::Rgb(0, 255, 128),   // green-cyan
        5 => Color::Rgb(180, 180, 180), // light gray
        _ => Color::Rgb(128, 128, 128), // gray
    };
    Style::default()
        .fg(color)
        .add_modifier(Modifier::BOLD)
}

fn syntect_fg_to_ratatui(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

fn syntect_bg_to_ratatui(color: syntect::highlighting::Color) -> Option<Color> {
    if color.a == 0 {
        None
    } else {
        Some(Color::Rgb(color.r, color.g, color.b))
    }
}

// ── span-wrapping for table cells ───────────────────────────────────

/// Wrap styled `Span`s to fit within `max_width` (in terminal columns).
fn wrap_spans(spans: &[Span<'static>], max_width: usize) -> Vec<Vec<Span<'static>>> {
    if max_width < 2 {
        return vec![spans.to_vec()];
    }

    let total: usize = spans
        .iter()
        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
        .sum();
    if total <= max_width {
        return vec![spans.to_vec()];
    }

    let mut lines: Vec<Vec<Span<'static>>> = vec![vec![]];
    let mut line_w = 0usize;

    for span in spans {
        let text = span.content.as_ref();
        let style = span.style;
        let mut pos = 0;

        while pos < text.len() {
            let avail = max_width.saturating_sub(line_w);
            if avail == 0 {
                lines.push(vec![]);
                line_w = 0;
                continue;
            }

            let end = find_fit(&text[pos..], avail);
            if end == 0 {
                // Force at least one character
                let c = text[pos..].chars().next().unwrap();
                let c_byte = c.len_utf8();
                let seg = &text[pos..pos + c_byte];
                lines.last_mut().unwrap().push(Span::styled(seg.to_string(), style));
                line_w += UnicodeWidthStr::width(seg);
                pos += c_byte;
            } else {
                let seg = &text[pos..pos + end];
                lines.last_mut().unwrap().push(Span::styled(seg.to_string(), style));
                line_w += UnicodeWidthStr::width(seg);
                pos += end;

                if pos < text.len() && line_w >= max_width {
                    lines.push(vec![]);
                    line_w = 0;
                    if text.as_bytes().get(pos) == Some(&b' ') {
                        pos += 1;
                    }
                }
            }
        }
    }

    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(vec![]);
    }
    lines
}

/// Find the longest prefix of `text` that fits within `max_width`,
/// preferring word boundaries (spaces). Returns byte offset.
fn find_fit(text: &str, max_width: usize) -> usize {
    if text.is_empty() || max_width == 0 {
        return 0;
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.len();
    }

    let mut last_space = 0usize;
    let mut width = 0usize;

    for (i, c) in text.char_indices() {
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if width + cw > max_width {
            return if last_space > 0 { last_space } else { i };
        }
        width += cw;
        if c == ' ' {
            last_space = i + c.len_utf8();
        }
    }

    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let md = r#"# Hello

This is a **test**.

- item 1
- item 2

```rust
fn main() {}
```
"#;
        let lines = render(md).unwrap();
        assert!(!lines.is_empty(), "should produce at least some lines");

        // Check that heading marker appears
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("#"), "should contain heading markers");
        assert!(all_text.contains("Hello"), "should contain the heading text");
        assert!(all_text.contains("test"), "should contain paragraph text");
        assert!(all_text.contains("•"), "should contain bullet points");
        assert!(all_text.contains("item 1"), "should contain list items");
        assert!(all_text.contains("fn main()"), "should contain code block content");
    }

    #[test]
    fn test_render_empty() {
        let lines = render("").unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_render_inline_styles() {
        let md = "This is **bold**, *italic*, `code`, and ~~strikethrough~~.";
        let lines = render(md).unwrap();
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("bold"));
        assert!(all_text.contains("italic"));
        assert!(all_text.contains("code"));
        assert!(all_text.contains("strikethrough"));
    }

    #[test]
    fn test_render_blockquote() {
        let md = "> A wise quote.\n> \n> — Someone";
        let lines = render(md).unwrap();
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("wise quote"), "blockquote text should appear");
        assert!(all_text.contains("Someone"), "blockquote attribution should appear");
    }

    #[test]
    fn test_render_table() {
        let md = "\
| Name    | Age | City     |
\
|---------|-----|----------|
\
| Alice   | 30  | Tokyo    |
\
| Bob     | 25  | New York |
\
";
        let lines = render(md).unwrap();
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("Alice"), "table body text should appear");
        assert!(all_text.contains("Name"), "table header should appear");
        assert!(all_text.contains("Tokyo"), "table data should appear");
        assert!(all_text.contains("Bob"), "table data should appear");
    }

    #[test]
    fn test_render_thematic_break() {
        let md = "Before\n\n---\n\nAfter";
        let lines = render(md).unwrap();
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("Before"));
        assert!(all_text.contains("After"));
    }
}

fn syntect_font_style_to_ratatui(font_style: SyntectFontStyle) -> Modifier {
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
