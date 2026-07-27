use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

pub fn wrap_spans(spans: &[Span<'static>], max_width: usize) -> Vec<Vec<Span<'static>>> {
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
