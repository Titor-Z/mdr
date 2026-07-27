mod blocks;
mod ctx;
mod inline;
mod style;
mod wrap;

pub use ctx::LinkInfo;

use anyhow::Result;
use comrak::{parse_document, Arena, ComrakOptions};
use ratatui::text::Line;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

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
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let root = parse_document(&arena, content, &options);

    let syn_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    let content_width = viewport_width.saturating_sub(2).max(20);

    let mut ctx = ctx::RenderCtx {
        lines: Vec::new(),
        list_indent: Vec::new(),
        syn_set: &syn_set,
        theme_set: &theme_set,
        viewport_width: content_width,
    };

    blocks::render_node(root, &mut ctx, 0);
    Ok(ctx.lines)
}

/// Extract links from rendered lines.
/// Our renderer formats links as `text [url]`, so we scan for `[url]` patterns.
pub fn extract_links(lines: &[Line<'static>]) -> Vec<LinkInfo> {
    let mut links = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let text = line.to_string();
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

    #[test]
    fn test_render_frontmatter() {
        let md = r#"---
title: My Document
date: 2024-01-01
tags: [rust, markdown]
---

# Hello"#;
        let lines = render(md).unwrap();
        let all_text: String = lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("title: My Document"), "frontmatter should appear");
        assert!(all_text.contains("date: 2024-01-01"), "frontmatter date should appear");
        assert!(all_text.contains("tags: [rust, markdown]"), "frontmatter tags should appear");
    }
}
