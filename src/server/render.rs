use comrak::{markdown_to_html_with_plugins, ComrakOptions, Plugins};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use serde::{Deserialize, Serialize};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

/// Metadata parsed from YAML frontmatter.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DocumentMeta {
    pub categories: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}


/// A heading item for the table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub level: usize,
    pub text: String,
    pub slug: String,
}

/// Map common language aliases to syntect-compatible names.
fn map_lang(lang: &str) -> &str {
    match lang {
        "ts" | "typescript" => "js",
        "py" => "python",
        "sh" => "bash",
        other => other,
    }
}

/// Render code with dual themes (Shiki-style — each span has --shiki-light and --shiki-dark).
fn highlight_dual_theme(code: &str, lang: &str) -> String {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let light_theme = &ts.themes["InspiredGitHub"];
    let dark_theme = &ts.themes["base16-ocean.dark"];

    let syn = ss.find_syntax_by_token(map_lang(lang))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut light_hl = HighlightLines::new(syn, light_theme);
    let mut dark_hl = HighlightLines::new(syn, dark_theme);

    let mut out = String::from("<pre class=\"shiki shiki-themes github-light github-dark\"><code>");

    for line in LinesWithEndings::from(code) {
        out.push_str("<span class=\"line\">");
        let light_regions = light_hl.highlight_line(line, &ss).unwrap_or_default();
        let dark_regions = dark_hl.highlight_line(line, &ss).unwrap_or_default();

        for (lr, dr) in light_regions.iter().zip(dark_regions.iter()) {
            let (light_style, light_text) = lr;
            let (dark_style, _dark_text) = dr;
            let lc = light_style.foreground;
            let dc = dark_style.foreground;
            out.push_str(&format!(
                "<span style=\"--shiki-light:#{:02x}{:02x}{:02x};--shiki-dark:#{:02x}{:02x}{:02x};\">{}</span>",
                lc.r, lc.g, lc.b, dc.r, dc.g, dc.b, esc_html(light_text)
            ));
        }
        out.push_str("</span>");
    }

    out.push_str("</code></pre>");
    out
}

/// Strip YAML frontmatter and return the document body only.
pub fn strip_frontmatter(content: &str) -> String {
    parse_frontmatter_meta(content).1
}

/// Parse YAML frontmatter into metadata, returning (meta, body_without_frontmatter).
fn parse_frontmatter_meta(content: &str) -> (DocumentMeta, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") {
        return (DocumentMeta::default(), content.to_string());
    }

    let after_opener = &trimmed[4..];
    let close_pos = after_opener.find("\n---\n");
    let (yaml, body) = match close_pos {
        Some(p) => (&after_opener[..p], after_opener[p + 5..].to_string()),
        None => return (DocumentMeta::default(), content.to_string()),
    };

    let mut meta = DocumentMeta::default();

    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line == "---" {
            continue;
        }
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_lowercase();
            let val = line[colon + 1..].trim();
            match key.as_str() {
                "categories" | "category" => {
                    meta.categories = parse_yaml_list(val);
                }
                "tags" => {
                    if meta.categories.is_empty() {
                        meta.categories = parse_yaml_list(val);
                    }
                }
                "date" | "created" | "create" => {
                    if meta.created.is_none() {
                        meta.created = Some(clean_date(val));
                    }
                }
                "updated" | "update" | "lastmod" | "modified" => {
                    meta.updated = Some(clean_date(val));
                }
                _ => {}
            }
        }
    }

    (meta, body)
}

fn parse_yaml_list(val: &str) -> Vec<String> {
    let val = val.trim();
    if val.starts_with('[') && val.ends_with(']') {
        val[1..val.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if val.starts_with('-') {
        // Multi-line list (won't be hit since we process line-by-line)
        vec![val.trim_start_matches('-').trim().to_string()]
    } else {
        vec![val.to_string()]
    }
}

fn clean_date(s: &str) -> String {
    s.trim().trim_matches('"').trim_matches('\'').to_string()
}

/// Render Markdown to HTML with syntax highlighting, returning (html, metadata).
pub fn markdown_to_html(content: &str) -> (String, DocumentMeta) {
    let content = preprocess_containers(content);
    let (meta, body) = parse_frontmatter_meta(&content);

    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    opts.extension.tagfilter = true;
    opts.render.unsafe_ = true;

    // Use SyntectAdapter — will post-process the output to add dual-theme
    let adapter = SyntectAdapterBuilder::new()
        .theme("base16-ocean.dark")
        .build();

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = markdown_to_html_with_plugins(&body, &opts, &plugins);

    // Post-process: convert single-theme <pre> blocks to dual-theme Shiki style
    let html = convert_to_dual_theme(&html);

    // Wrap standalone code blocks + heading anchors
    let html = wrap_code_blocks(&html);
    let html = wrap_tables(&html);
    let html = add_header_anchors(&html);
    (html, meta)
}

/// Replace <pre style="background-color:#..."> with dual-theme Shiki-style <pre class="shiki">.
fn convert_to_dual_theme(html: &str) -> String {
    // Extract code from each comrak-generated <pre> and re-render with dual themes
    let mut out = String::with_capacity(html.len() + 2048);
    let mut rest = html;

    while let Some(ps) = rest.find("<pre") {
        out.push_str(&rest[..ps]);
        rest = &rest[ps..];

        // Find </pre>
        let end = match rest.find("</pre>") {
            Some(e) => e + 6,
            None => { out.push_str(rest); break; }
        };

        let block = &rest[..end];
        rest = &rest[end..];

        // Extract language from class="language-xxx"
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

        // Extract PLAIN TEXT code (strip HTML tags, decode entities)
        if let Some(cs) = block.find("<code") {
            let code_tag_end = block[cs..].find('>').map(|p| cs + p + 1).unwrap_or(0);
            let code_end_tag = format!("</code>");
            if let Some(ce) = block[code_tag_end..].find(&code_end_tag) {
                let inner = &block[code_tag_end..code_tag_end + ce];
                // Strip all HTML tags to get plain text
                let mut plain = String::with_capacity(inner.len());
                let mut in_tag = false;
                for ch in inner.chars() {
                    match ch {
                        '<' => in_tag = true,
                        '>' => in_tag = false,
                        _ if !in_tag => plain.push(ch),
                        _ => {}
                    }
                }
                let decoded = htmldecode(&plain);
                let highlighted = highlight_dual_theme(&decoded, lang.unwrap_or("text"));
                out.push_str(&highlighted);
                continue;
            }
        }

        // Fallback: keep original
        out.push_str(block);
    }

    out.push_str(rest);
    out
}

// ── container preprocessing ──────────────────────────────────────────

/// Pre-process markdown to replace `:::` containers with raw HTML.
fn preprocess_containers(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 2048);
    let mut rest = content;

    while let Some(start) = rest.find("\n:::") {
        if start != 0 && &rest[start - 1..start] != "\n" {
            out.push_str(&rest[..=start]);
            rest = &rest[start + 1..];
            continue;
        }
        out.push_str(&rest[..start]);
        rest = &rest[start + 1..];

        let eol = rest.find('\n').unwrap_or(rest.len());
        let header = &rest[..eol];
        rest = &rest[eol + 1..];

        let close_marker = rest.find("\n:::");
        let close_pos = match close_marker {
            Some(p) => p,
            None => { out.push_str(rest); break; }
        };
        let block = &rest[..close_pos];
        rest = &rest[close_pos + 4..];

        let header_trimmed = header.trim_start_matches(':').trim();
        let parts: Vec<&str> = header_trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        let ctype = parts.first().unwrap_or(&"").trim();
        let ctitle = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match ctype {
            "code-group" => {
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
                    let highlighted = highlight_dual_theme(code, &lang);
                    blocks_html.push_str(&format!("<div class=\"code-block{}\"><div class=\"language-{}\"><button class=\"copy\" title=\"复制代码\"></button>{}</div></div>", active_class, lang, highlighted));
                    tab_idx += 1;
                    pos = next_pos;
                }
                out.push_str(&format!("<div class=\"vp-code-group\"><div class=\"tabs\">{}</div><div class=\"blocks\">{}</div></div>", tabs_html, blocks_html));
            }
            "info" | "tip" | "warning" | "danger" | "caution" | "important" | "note" => {
                let title_html = if !ctitle.is_empty() { format!("<p class=\"custom-block-title\">{}</p>", ctitle) } else { String::new() };
                out.push_str(&format!("<div class=\"custom-block {}\">{}{}</div>", ctype, title_html, block));
            }
            "details" => {
                let summary = if !ctitle.is_empty() { ctitle } else { "详细信息" };
                out.push_str(&format!("<details class=\"custom-block details\"><summary>{}</summary>{}</details>", summary, block));
            }
            _ => {}
        }
    }

    out.push_str(rest);
    out
}

// ── code block wrapper ──────────────────────────────────────────────

/// Wrap standalone <pre> blocks in VitePress-style structure.
fn wrap_code_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 2048);
    let mut rest = html;

    while let Some(ps) = rest.find("<pre") {
        out.push_str(&rest[..ps]);
        rest = &rest[ps..];

        if rest.starts_with("<pre data-cg=\"1\"") {
            let end = match rest.find("</pre>") {
                Some(e) => e + 6,
                None => { out.push_str(rest); break; }
            };
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }

        let end = match rest.find("</pre>") {
            Some(e) => e + 6,
            None => { out.push_str(rest); break; }
        };
        let block = &rest[..end];
        rest = &rest[end..];

        // Extract language
        let lang = if let Some(cs) = block.find("class=\"language-") {
            let after = &block[cs + 16..];
            if let Some(ce) = after.find('"') { Some(&after[..ce]) } else { None }
        } else { None };
        let lang_label = lang.unwrap_or("text");

        out.push_str(&format!(
            "<div class=\"vp-code-block-title\"><div class=\"vp-code-block-title-bar\"><span class=\"vp-code-block-title-text\">{}</span></div><div class=\"language-{}\"><button class=\"copy\" title=\"复制代码\"></button>{}</div></div>",
            lang_label, lang_label, block
        ));
    }

    out.push_str(rest);
    out
}

// ── table wrapper ──────────────────────────────────────────────────

/// Wrap <table> in a scrollable div to prevent layout breakage.
fn wrap_tables(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 1024);
    let mut rest = html;

    while let Some(ts) = rest.find("<table") {
        out.push_str(&rest[..ts]);
        rest = &rest[ts..];

        // Find closing </table>
        let end = match rest.find("</table>") {
            Some(e) => e + 8,
            None => { out.push_str(rest); break; }
        };

        let table = &rest[..end];
        rest = &rest[end..];

        out.push_str("<div class=\"table-wrapper\">");
        out.push_str(table);
        out.push_str("</div>");
    }

    out.push_str(rest);
    out
}

// ── heading anchors ─────────────────────────────────────────────────

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn htmldecode(s: &str) -> String {
    s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&quot;", "\"").replace("&#39;", "'")
}

fn add_header_anchors(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 1024);
    let mut rest = html;
    while let Some(hs) = rest.find("<h") {
        out.push_str(&rest[..hs]);
        rest = &rest[hs..];
        if rest.len() < 5 { out.push_str(rest); break; }
        if rest.len() < 4 { out.push_str(rest); break; }
        let level_char = rest.as_bytes()[2] as char;
        if level_char != '1' && level_char != '2' && level_char != '3' && level_char != '4' {
            out.push_str(&rest[..1]); rest = &rest[1..]; continue;
        }
        let cb = match rest.find('>') { Some(p) => p, None => { out.push_str(rest); break; } };
        let tag_name = &rest[1..cb];
        let bare_tag: &str = if let Some(sp) = tag_name.find(' ') { &tag_name[..sp] } else { tag_name };
        rest = &rest[cb + 1..];
        let ct = format!("</{}>", bare_tag);
        let end = match rest.find(&ct) { Some(p) => p, None => { out.push_str(rest); break; } };
        let text = &rest[..end];
        let slug = text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' || *c == '.')
            .collect::<String>().to_lowercase().trim().replace(' ', "-");
        out.push_str(&format!("<{} id=\"{}\"><a class=\"header-anchor\" href=\"#{}\" aria-hidden=\"true\">#</a>{}</{}>", tag_name, slug, slug, text, bare_tag));
        rest = &rest[end + ct.len()..];
    }
    out.push_str(rest);
    out
}

// ── ToC extraction ──────────────────────────────────────────────────

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
        if level == 1 {
            // skip h1 — usually the document title
        } else {
            let text = collect_text(node);
            let slug = text.chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' || *c == '.')
                .collect::<String>().to_lowercase().trim().replace(' ', "-");
            items.push(TocItem { level, text, slug });
        }
    }
    for child in node.children() { walk_headings(child, items); }
}

fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    let data = node.data.borrow();
    if let comrak::nodes::NodeValue::Text(t) = &data.value { return t.to_string(); }
    let mut out = String::new();
    for child in node.children() { out.push_str(&collect_text(child)); }
    out
}

// ── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let (html, meta) = markdown_to_html("# Hello\n\nThis is **bold** text.");
        assert!(html.contains("<h1>") || html.contains("<h1 "));
        assert!(html.contains("<strong>"));
        assert!(meta.categories.is_empty());
    }

    #[test]
    fn test_table_rendering() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let (html, _) = markdown_to_html(md);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_task_list() {
        let md = "- [ ] Todo\n- [x] Done\n";
        let (html, _) = markdown_to_html(md);
        assert!(html.contains("checkbox") || html.contains("checked"));
    }

    #[test]
    fn test_dual_theme_highlighting() {
        let html = highlight_dual_theme("fn main() {}", "rust");
        assert!(html.contains("shiki"));
        assert!(html.contains("--shiki-light"));
        assert!(html.contains("--shiki-dark"));
        assert!(html.contains("class=\"line\""));
    }

    #[test]
    fn test_header_anchors() {
        let (html, _) = markdown_to_html("## Getting Started\n");
        assert!(html.contains("header-anchor"));
        assert!(html.contains("id=\"getting-started\""));
    }

    #[test]
    fn test_extract_toc() {
        let toc = extract_toc("# A\n\n## B\n\n### C\n");
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].level, 2);
        assert_eq!(toc[0].text, "B");
        assert_eq!(toc[1].level, 3);
        assert_eq!(toc[1].text, "C");
    }

    #[test]
    fn test_preprocess_containers() {
        let md = "# Test\n\n::: info\nInfo box\n:::\n";
        let result = preprocess_containers(md);
        assert!(result.contains("custom-block info"));
    }

    #[test]
    fn test_frontmatter_meta() {
        let md = r#"---
categories: [rust, tools]
date: 2024-01-01
updated: 2024-06-15
---

# Hello"#;
        let (_, meta) = markdown_to_html(md);
        assert_eq!(meta.categories, vec!["rust", "tools"]);
        assert_eq!(meta.created, Some("2024-01-01".to_string()));
        assert_eq!(meta.updated, Some("2024-06-15".to_string()));
    }

    #[test]
    fn test_frontmatter_meta_tags_fallback() {
        let md = r#"---
tags: [cli, markdown]
date: 2024-01-01
---

# Hello"#;
        let (_, meta) = markdown_to_html(md);
        assert_eq!(meta.categories, vec!["cli", "markdown"]);
        assert_eq!(meta.created, Some("2024-01-01".to_string()));
    }

    #[test]
    fn test_frontmatter_meta_created_fallback() {
        let md = r#"---
create: 2024-01-01
---

# Hello"#;
        let (_, meta) = markdown_to_html(md);
        assert_eq!(meta.created, Some("2024-01-01".to_string()));
        assert!(meta.updated.is_none());
    }
}
