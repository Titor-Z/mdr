use crate::render::ctx::RenderCtx;
use comrak::nodes::{AstNode, NodeValue};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

pub fn collect_inline<'a>(node: &'a AstNode<'a>, ctx: &RenderCtx<'a>, spans: &mut Vec<Span<'static>>) {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Text(t) => {
            spans.push(Span::raw(t.clone()));
        }
        NodeValue::SoftBreak => {}
        NodeValue::LineBreak => {}
        NodeValue::Code(c) => {
            spans.push(Span::styled(
                c.literal.clone(),
                Style::default()
                    .bg(Color::Rgb(60, 60, 60))
                    .fg(Color::Rgb(203, 170, 111)),
            ));
        }
        NodeValue::Emph => {
            for span in collect_inline_children(node, ctx) {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::ITALIC),
                ));
            }
        }
        NodeValue::Strong => {
            for span in collect_inline_children(node, ctx) {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::BOLD),
                ));
            }
        }
        NodeValue::Strikethrough => {
            for span in collect_inline_children(node, ctx) {
                spans.push(Span::styled(
                    span.content.to_string(),
                    span.style.add_modifier(Modifier::CROSSED_OUT),
                ));
            }
        }
        NodeValue::Link(link) => {
            let mut child_spans = collect_inline_children(node, ctx);
            for span in &mut child_spans {
                span.style = span.style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED);
            }
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

fn collect_inline_children<'a>(node: &'a AstNode<'a>, ctx: &RenderCtx<'a>) -> Vec<Span<'static>> {
    let mut spans = vec![];
    for child in node.children() {
        collect_inline(child, ctx, &mut spans);
    }
    spans
}
