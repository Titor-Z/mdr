use crate::render::ctx::RenderCtx;
use crate::render::inline::collect_inline;
use crate::render::style;
use crate::render::wrap::wrap_spans;
use comrak::nodes::{AstNode, ListType, NodeValue};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use unicode_width::UnicodeWidthStr;

pub fn render_node<'a>(node: &'a AstNode<'a>, ctx: &mut RenderCtx<'a>, depth: usize) {
    let data = node.data.borrow();

    match &data.value {
        NodeValue::Document => {
            for child in node.children() {
                render_node(child, ctx, depth);
            }
        }

        NodeValue::FrontMatter(fm) => {
            let dim = Style::default()
                .fg(Color::Rgb(120, 120, 135))
                .add_modifier(Modifier::DIM);
            ctx.lines.push(Line::from(""));
            for line in fm.lines().filter(|l| *l != "---") {
                ctx.lines.push(Line::from(Span::styled(line.to_string(), dim)));
            }
            ctx.lines.push(Line::from(""));
        }

        NodeValue::Heading(h) => {
            let h_style = style::heading_style(h.level);
            let mut spans = vec![];

            for child in node.children() {
                collect_inline(child, ctx, &mut spans);
            }

            let prefix = "#".repeat(h.level as usize);
            let marker = Span::styled(format!("{} ", prefix), h_style);
            spans.insert(0, marker);

            if !ctx.lines.is_empty() {
                ctx.lines.push(Line::from(""));
            }
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

            ctx.lines.push(Line::from(""));

            for line in code.lines() {
                let spans = if let Some(ref mut hl) = highlighter {
                    let ranges = hl.highlight_line(line, ctx.syn_set).unwrap();
                    ranges
                        .into_iter()
                        .map(|(syntect_color, text)| {
                            let fg = style::syntect_fg_to_ratatui(syntect_color.foreground);
                            let bg = style::syntect_bg_to_ratatui(syntect_color.background);
                            let modifier = style::syntect_font_style_to_ratatui(syntect_color.font_style);
                            Span::styled(
                                text.to_string(),
                                Style::default()
                                    .fg(fg)
                                    .bg(bg.unwrap_or(Color::Rgb(43, 48, 59)))
                                    .add_modifier(modifier),
                            )
                        })
                        .collect()
                } else {
                    vec![Span::styled(
                        line.to_string(),
                        Style::default().bg(Color::Rgb(43, 48, 59)),
                    )]
                };
                ctx.lines.push(Line::from(spans));
            }

            ctx.lines.push(Line::from(""));
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

            let mut natural = vec![0usize; num_cols];
            for row in &raw_rows {
                for (ci, cell) in row.iter().enumerate() {
                    let w: usize = cell.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
                    natural[ci] = natural[ci].max(w);
                }
            }

            let indent = ctx.list_indent.last().copied().unwrap_or(0);
            let border_chars = 2 + num_cols.saturating_sub(1);
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

            let mut wrapped_rows: Vec<Vec<Vec<Vec<Span<'static>>>>> = Vec::new();
            for row in &raw_rows {
                let mut wrapped_cells = Vec::new();
                for (ci, cell) in row.iter().enumerate() {
                    let cw = col_widths.get(ci).copied().unwrap_or(10);
                    wrapped_cells.push(wrap_spans(cell, cw));
                }
                wrapped_rows.push(wrapped_cells);
            }

            let border = Style::default().fg(Color::Rgb(100, 100, 110));
            let header_style = Style::default().add_modifier(Modifier::BOLD);

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
            ctx.lines.push(mk_border("┌", "┬", "┐", "┬"));

            for (ri, wrapped_cells) in wrapped_rows.iter().enumerate() {
                let max_lines = wrapped_cells.iter().map(|c| c.len()).max().unwrap_or(1).max(1);

                for line_i in 0..max_lines {
                    let mut spans: Vec<Span<'static>> = vec![];
                    if indent > 0 {
                        spans.push(Span::raw(" ".repeat(indent)));
                    }
                    spans.push(Span::styled("│", border));

                    for (ci, cell_lines) in wrapped_cells.iter().enumerate() {
                        let cw = col_widths[ci];
                        spans.push(Span::raw(" "));

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
                            spans.push(Span::raw(" ".repeat(pad + 1)));
                        } else {
                            spans.push(Span::raw(" ".repeat(cw + 1)));
                        }

                        spans.push(Span::styled("│", border));
                    }

                    ctx.lines.push(Line::from(spans));
                }

                if ri == 0 && wrapped_rows.len() > 1 {
                    ctx.lines.push(mk_border("├", "┼", "┤", "┼"));
                } else if ri > 0 && ri + 1 < wrapped_rows.len() {
                    ctx.lines.push(mk_border("├", "┼", "┤", "┼"));
                }
            }

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
    node: &'a AstNode<'a>,
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
                    let bullet_prefix_width = if first {
                        indent.saturating_sub(2) + 2
                    } else {
                        indent
                    };
                    let max_w = ctx.viewport_width.saturating_sub(bullet_prefix_width).max(10);
                    let wrapped = wrap_spans(&content_spans, max_w);

                    for (wi, line_spans) in wrapped.iter().enumerate() {
                        let mut full = vec![];
                        if wi == 0 && first {
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
                render_node(child, ctx, 0);
                first = false;
            }
        }
    }
}
