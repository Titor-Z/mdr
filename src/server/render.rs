use comrak::{markdown_to_html_with_plugins, ComrakOptions, Plugins};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use serde::{Deserialize, Serialize};

/// A heading item for the table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub level: usize,
    pub text: String,
    pub slug: String,
}

/// Render Markdown to HTML with syntax highlighting and heading anchors.
pub fn markdown_to_html(content: &str) -> String {
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = false;

    // SyntectAdapter with Solarized (dark) — 接近 VitePress github-dark 配色
    let adapter = SyntectAdapterBuilder::new()
        .theme("Solarized (dark)")
        .build();

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = markdown_to_html_with_plugins(content, &opts, &plugins);

    // VitePress-style code block wrappers + heading anchors
    let html = wrap_code_blocks(&html);
    add_header_anchors(&html)
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

fn walk_headings<'a>(node: &'a comrak::nodes::AstNode<'a>, items: &mut Vec<TocItem>) {
    let data = node.data.borrow();
    if let comrak::nodes::NodeValue::Heading(heading) = &data.value {
        let level = heading.level as usize;
        let text = collect_text(node);
        let slug = heading_slug(&text);
        items.push(TocItem { level, text, slug });
    }
    for child in node.children() {
        walk_headings(child, items);
    }
}

fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    let data = node.data.borrow();
    if let comrak::nodes::NodeValue::Text(t) = &data.value {
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

/// Add VitePress-style anchor links to h1-h4 headings.
/// Wrap <pre><code> blocks in VitePress-style structure with language label and copy button.
fn wrap_code_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 2048);
    let mut rest = html;

    while let Some(ps) = rest.find("<pre") {
        out.push_str(&rest[..ps]);
        rest = &rest[ps..];

        // Find the closing </pre>
        let end = rest.find("</pre>");
        let end = match end {
            Some(e) => e + 6,
            None => { out.push_str(rest); break; }
        };

        let block = &rest[..end];
        rest = &rest[end..];

        // Extract language from <code class="language-xxx">
        let lang = if let Some(cs) = block.find("class=\"language-") {
            let after = &block[cs + 16..];
            if let Some(ce) = after.find('"') {
                Some(&after[..ce])
            } else {
                None
            }
        } else {
            None
        };

        let lang_label = lang.unwrap_or("");

        // Remove inline style from <pre> (syntect adds background-color)
        let block_clean = block.replace(" style=\"background-color:#002b36;\"", "");

        // VitePress wrapper — 标题栏 + 代码区
        out.push_str(&format!(
            r##"<div class="vp-code-block-title"><div class="vp-code-block-title-bar"><span class="vp-code-block-title-text">{}</span></div><div class="language-{}"><button class="copy" title="复制代码"></button><span class="lang">{}</span>"##,
            lang_label, lang_label, lang_label
        ));
        out.push_str(&block_clean);
        out.push_str("</div></div>");
    }

    out.push_str(rest);
    out
}

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

        // rest starts at "<hN...>"
        if rest.len() < 4 {
            out.push_str(rest);
            break;
        }
        let level_char = rest.as_bytes()[2] as char;
        if level_char != '1' && level_char != '2' && level_char != '3' && level_char != '4' {
            out.push_str(&rest[..1]);
            rest = &rest[1..];
            continue;
        }

        let cb = match rest.find('>') {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        let tag_name = &rest[1..cb];
        let bare_tag: &str = if let Some(sp) = tag_name.find(' ') {
            &tag_name[..sp]
        } else {
            tag_name
        };

        rest = &rest[cb + 1..];

        let ct = format!("</{}>", bare_tag);
        let end = match rest.find(&ct) {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        let text = &rest[..end];
        let slug = heading_slug(text);

        out.push_str(&format!(
            "<{} id=\"{}\"><a class=\"header-anchor\" href=\"#{}\" aria-hidden=\"true\">#</a>{}</{}>",
            tag_name, slug, slug, text, bare_tag
        ));

        rest = &rest[end + ct.len()..];
    }

    out.push_str(rest);
    out
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

    #[test]
    fn test_extract_toc() {
        let toc = extract_toc("# A\n\n## B\n\n### C\n");
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "A");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[1].text, "B");
        assert_eq!(toc[2].level, 3);
        assert_eq!(toc[2].text, "C");
    }
}
