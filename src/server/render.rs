use comrak::ComrakOptions;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

/// Render Markdown to HTML with syntax highlighting.
pub fn markdown_to_html(content: &str) -> String {
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = false;

    // Set up syntax highlighting for <code> blocks via HTML post-processing
    let html = comrak::markdown_to_html_with_plugins(content, &opts, &comrak::Plugins::default());

    // Apply syntax highlighting to <pre><code> blocks
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    highlight_code_blocks(&html, &ss, theme)
}

/// Post-process HTML to add syntax highlighting to <pre><code> blocks.
fn highlight_code_blocks(html: &str, ss: &SyntaxSet, theme: &syntect::highlighting::Theme) -> String {
    let mut result = String::with_capacity(html.len() + 4096);
    let mut remaining = html;

    while let Some(pre_start) = remaining.find("<pre><code") {
        // Push everything before <pre><code
        result.push_str(&remaining[..pre_start]);
        remaining = &remaining[pre_start..];

        // Find end of opening tag
        let tag_end = remaining.find(">").unwrap_or(0);
        let after_tag = &remaining[tag_end + 1..];

        // Find closing </code></pre>
        let close = after_tag.find("</code></pre>");
        let (code_text, rest) = if let Some(pos) = close {
            (&after_tag[..pos], &after_tag[pos + 13..])
        } else {
            break;
        };

        // Decode HTML entities for highlighting
        let decoded = htmldecode(code_text);

        // Detect language from class attribute
        let class_attr = &remaining[..tag_end];
        let lang = if let Some(cstart) = class_attr.find("class=\"language-") {
            let after = &class_attr[cstart + 17..];
            if let Some(cend) = after.find('"') {
                Some(&after[..cend])
            } else {
                None
            }
        } else {
            None
        };

        let highlighted = if let Some(lang) = lang {
            if let Some(syntax) = ss.find_syntax_by_token(lang) {
                highlighted_html_for_string(&decoded, ss, syntax, theme)
                    .unwrap_or_else(|_| format!("<code>{}</code>", esc_html(code_text)))
            } else {
                format!("<code>{}</code>", esc_html(code_text))
            }
        } else {
            format!("<code>{}</code>", esc_html(code_text))
        };

        // Wrap with <pre>
        result.push_str("<pre style=\"background:#1b2b34;color:#cdd3de;padding:16px;border-radius:8px;overflow-x:auto\">");
        result.push_str(&highlighted);
        result.push_str("</pre>");
        remaining = rest;
    }

    result.push_str(remaining);
    result
}

fn htmldecode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let html = markdown_to_html("# Hello\n\nThis is **bold** text.");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn test_table_rendering() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_task_list() {
        let md = "- [ ] Todo\n- [x] Done\n";
        let html = markdown_to_html(md);
        assert!(html.contains("type=\"checkbox\"") || html.contains("checked"));
    }

    #[test]
    fn test_syntax_highlighting() {
        let md = "```rust\nfn main() {}\n```\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<pre"));
        assert!(html.contains("fn")); // syntax highlighted output should still contain the code
    }
}
