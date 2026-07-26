use comrak::ComrakOptions;
use comrak::nodes::{AstNode, NodeValue};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use serde::{Deserialize, Serialize};

/// A heading item for the table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub level: usize,
    pub text: String,
    pub slug: String,
}

/// Extract table of contents from markdown.
pub fn extract_toc(content: &str) -> Vec<TocItem> {
    let arena = comrak::Arena::new();
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = false;

    let root = comrak::parse_document(&arena, content, &opts);
    let mut items = Vec::new();
    walk_headings(root, &mut items);
    items
}

fn walk_headings<'a>(node: &'a AstNode<'a>, items: &mut Vec<TocItem>) {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Heading(heading) => {
            let level = heading.level as usize;
            // Collect text content from inline children
            let text = collect_text(node);
            let slug = heading_slug(&text);
            items.push(TocItem { level, text, slug });
        }
        _ => {}
    }
    for child in node.children() {
        walk_headings(child, items);
    }
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let data = node.data.borrow();
    if let NodeValue::Text(t) = &data.value {
        return t.to_string();
    }
    let mut out = String::new();
    for child in node.children() {
        out.push_str(&collect_text(child));
    }
    out
}

fn heading_slug(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' || *c == '.')
        .collect::<String>()
        .to_lowercase()
        .trim()
        .replace(' ', "-")
}

/// Render Markdown to HTML with syntax highlighting and heading anchors.
pub fn markdown_to_html(content: &str) -> String {
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = false;

    let html = comrak::markdown_to_html_with_plugins(content, &opts, &comrak::Plugins::default());

    // VitePress-style heading anchors
    let html = add_header_anchors(&html);

    // Syntax highlighting
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];
    highlight_code_blocks(&html, &ss, theme)
}

/// Add VitePress-style anchor links to h1-h4 headings.
fn add_header_anchors(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 1024);
    let mut rest = html;

    while let Some(hs) = rest.find("<h") {
        out.push_str(&rest[..hs]);
        rest = &rest[hs..];

        if rest.len() < 5 {
            out.push_str(rest);
            break;
        }

        // rest starts at "<hN...", so level digit is at index 2
        if rest.len() < 4 {
            out.push_str(rest);
            break;
        }
        let level_char = rest.as_bytes()[2] as char; // '1'..'4'
        if level_char != '1' && level_char != '2' && level_char != '3' && level_char != '4' {
            out.push_str(&rest[..1]);
            rest = &rest[1..];
            continue;
        }

        // Find closing '>' of the opening tag
        let cb = match rest.find('>') {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        // Extract the tag name (e.g. "h2") from "<h2...>"
        let tag_name = &rest[1..cb]; // e.g. "h2" or "h2 class=..."
        let bare_tag: &str = if let Some(sp) = tag_name.find(' ') {
            &tag_name[..sp]
        } else {
            tag_name
        };

        rest = &rest[cb + 1..];

        // Find closing tag
        let ct = format!("</{}>", bare_tag);
        let end = match rest.find(&ct) {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        let text = &rest[..end];

        let slug = heading_slug(text);

        // Rewrite heading: <h2 id="slug"><a class="header-anchor" href="#slug">#</a>text</h2>
        out.push_str(&format!(
            "<{} id=\"{}\"><a class=\"header-anchor\" href=\"#{}\" aria-hidden=\"true\">#</a>{}</{}>",
            tag_name, slug, slug, text, bare_tag
        ));

        rest = &rest[end + ct.len()..]; // skip past </hN>
    }

    out.push_str(rest);
    out
}

/// Apply syntax highlighting to <pre><code> blocks.
fn highlight_code_blocks(html: &str, ss: &SyntaxSet, theme: &syntect::highlighting::Theme) -> String {
    let mut out = String::with_capacity(html.len() + 4096);
    let mut rest = html;

    while let Some(ps) = rest.find("<pre><code") {
        out.push_str(&rest[..ps]);
        rest = &rest[ps..];

        let te = match rest.find('>') {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };
        let after = &rest[te + 1..];

        let close = match after.find("</code></pre>") {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        let code = &after[..close];

        // Detect language from class attribute on the opening tag
        let attrs = &rest[..te];
        let lang = attrs
            .find("class=\"language-")
            .and_then(|cs| {
                let a = &attrs[cs + 17..];
                a.find('"').map(|ce| &a[..ce])
            });

        let decoded = htmldecode(code);

        let highlighted = if let Some(lang) = lang {
            if let Some(syn) = ss.find_syntax_by_token(lang) {
                highlighted_html_for_string(&decoded, ss, syn, theme)
                    .unwrap_or_else(|_| esc_html(code))
            } else {
                esc_html(code)
            }
        } else {
            esc_html(code)
        };

        out.push_str("<pre style=\"background:#1b2b34;color:#cdd3de;padding:16px;border-radius:8px;overflow-x:auto\">");
        out.push_str(&highlighted);
        out.push_str("</pre>");
        rest = &after[close + 13..];
    }

    out.push_str(rest);
    out
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let html = markdown_to_html("# Hello\n\nThis is **bold** text.");
        assert!(html.contains("<h1>") || html.contains("<h1 "));
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn test_table_rendering() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_task_list() {
        let md = "- [ ] Todo\n- [x] Done\n";
        let html = markdown_to_html(md);
        assert!(html.contains("checkbox") || html.contains("checked"));
    }

    #[test]
    fn test_syntax_highlighting() {
        let md = "```rust\nfn main() {}\n```\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<pre"));
        assert!(html.contains("fn"));
    }

    #[test]
    fn test_header_anchors() {
        let html = markdown_to_html("## Getting Started\n");
        assert!(html.contains("header-anchor"));
        assert!(html.contains("id=\"getting-started\""));
    }
}
