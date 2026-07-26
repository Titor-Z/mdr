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

/// Pre-process markdown to replace `:::` containers with raw HTML.
fn preprocess_containers(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 2048);
    let mut rest = content;

    while let Some(start) = rest.find("\n:::") {
        // Check it starts at a line boundary (after newline or at start)
        if start != 0 && &rest[start - 1..start] != "\n" {
            out.push_str(&rest[..=start]);
            rest = &rest[start + 1..];
            continue;
        }

        out.push_str(&rest[..start]);
        rest = &rest[start + 1..]; // skip the first \n, keep "::: ..."

        // Find the end of the opening line
        let eol = rest.find('\n').unwrap_or(rest.len());
        let header = &rest[..eol]; // e.g., "::: info" or "::: code-group"
        rest = &rest[eol + 1..];

        // Find closing :::
        let close_marker = rest.find("\n:::");
        let close_pos = match close_marker {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };

        let block = &rest[..close_pos];
        rest = &rest[close_pos + 4..]; // skip past "\n:::\n"

        // Parse type and optional title
        let header_trimmed = header.trim_start_matches(':').trim();
        let parts: Vec<&str> = header_trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        let ctype = parts.first().unwrap_or(&"").trim();
        let ctitle = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match ctype {
            "code-group" => {
                // Code group with tabs
                let mut tabs_html = String::new();
                let mut blocks_html = String::new();
                let mut tab_idx = 0;
                let mut pos = 0;

                while let Some(fs) = block[pos..].find("```") {
                    let fence_start = pos + fs;
                    let eol_fence = block[fence_start..].find('\n').unwrap_or(block.len() - fence_start);
                    let header = block[fence_start + 3..fence_start + eol_fence].trim().to_string();
                    let (lang, title) = if let Some(tb) = header.find(" [") {
                        (header[..tb].to_string(), header[tb + 2..].trim_end_matches(']').to_string())
                    } else {
                        (header.clone(), header)
                    };

                    let code_start = fence_start + eol_fence + 1;
                    let close_fence = block[code_start..].find("\n```");
                    let (code, next_pos) = match close_fence {
                        Some(c) => {
                            let code_end = code_start + c;
                            let next = code_end + 5;
                            let next = if next < block.len() && block.as_bytes()[next] == b'\n' { next + 1 } else { next };
                            (&block[code_start..code_end], next)
                        }
                        None => (&block[code_start..], block.len()),
                    };

                    let active_class = if tab_idx == 0 { " active" } else { "" };
                    tabs_html.push_str(&format!("<label class=\"tab{}\" data-title=\"{}\">{}</label>", active_class, title, title));

                    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
                    let ts = syntect::highlighting::ThemeSet::load_defaults();
                    let theme = &ts.themes["Solarized (dark)"];
                    let lang_token = match lang.as_str() { "ts" | "typescript" => "js", "py" => "python", "sh" => "bash", o => o };

                    let highlighted = if let Some(syn) = ss.find_syntax_by_token(lang_token) {
                        use syntect::easy::HighlightLines;
                        use syntect::html::append_highlighted_html_for_styled_line;
                        use syntect::util::LinesWithEndings;
                        let mut hl = HighlightLines::new(syn, theme);
                        let bg = theme.settings.background.unwrap_or(syntect::highlighting::Color::WHITE);
                        let mut o = String::new();
                        for line in LinesWithEndings::from(code) {
                            let regions = hl.highlight_line(line, &ss).unwrap_or_default();
                            append_highlighted_html_for_styled_line(&regions[..], syntect::html::IncludeBackground::IfDifferent(bg), &mut o).ok();
                        }
                        o
                    } else {
                        format!("<span style=\"color:#657b83;\">{}</span>", esc_html(code))
                    };

                    blocks_html.push_str(&format!("<div class=\"code-block{}\"><div class=\"language-{}\"><button class=\"copy\" title=\"复制代码\"></button><pre data-cg=\"1\"><code>{}</code></pre></div></div>", active_class, lang, highlighted));
                    tab_idx += 1;
                    pos = next_pos;
                }

                out.push_str(&format!("<div class=\"vp-code-group\"><div class=\"tabs\">{}</div><div class=\"blocks\">{}</div></div>", tabs_html, blocks_html));
            }

            "info" | "tip" | "warning" | "danger" | "caution" | "important" | "note" => {
                // Custom container
                let title_html = if !ctitle.is_empty() {
                    format!("<p class=\"custom-block-title\">{}</p>", ctitle)
                } else {
                    String::new()
                };
                out.push_str(&format!("<div class=\"custom-block {}\">{}{}</div>", ctype, title_html, block));
            }

            "details" => {
                let summary = if !ctitle.is_empty() { ctitle } else { "详细信息" };
                out.push_str(&format!("<details class=\"custom-block details\"><summary>{}</summary>{}</details>", summary, block));
            }

            _ => {
                // Unknown container, pass through as-is
                out.push_str(&format!(":::{}  {}", ctype, ctitle));
            }
        }
    }

    out.push_str(rest);
    out
}

/// Render Markdown to HTML with syntax highlighting and heading anchors.
pub fn markdown_to_html(content: &str) -> String {
    // Pre-process code groups
    let content = preprocess_containers(content);

    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = true;

    // SyntectAdapter with Solarized (dark) — 接近 VitePress github-dark 配色
    let adapter = SyntectAdapterBuilder::new()
        .theme("Solarized (dark)")
        .build();

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = markdown_to_html_with_plugins(&content, &opts, &plugins);

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
/// Wrap standalone <pre><code> blocks in VitePress-style structure.
/// Skips blocks already inside vp-code-group (code groups).
fn wrap_code_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 2048);
    let mut rest = html;

    while let Some(ps) = rest.find("<pre") {
        out.push_str(&rest[..ps]);
        rest = &rest[ps..];

        // Skip <pre> tags inside code groups (marked with data-cg="1")
        if rest.starts_with("<pre data-cg=\"1\"") {
            let end = match rest.find("</pre>") {
                Some(e) => e + 6,
                None => { out.push_str(rest); break; }
            };
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }

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

        let lang_label = lang.unwrap_or("text");

        // Remove inline style from <pre> (syntect adds background-color)
        let block_clean = block.replace(" style=\"background-color:#002b36;\"", "");

        // VitePress wrapper — 标题栏 + 代码区
        out.push_str(&format!(
            r##"<div class="vp-code-block-title"><div class="vp-code-block-title-bar"><span class="vp-code-block-title-text">{}</span></div><div class="language-{}"><button class="copy" title="复制代码"></button>"##,
            lang_label, lang_label
        ));
        out.push_str(&block_clean);
        out.push_str("</div></div>");
    }

    out.push_str(rest);
    out
}

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
