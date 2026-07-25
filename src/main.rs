use std::time::SystemTime;

use clap::Parser;
use anyhow::Result;

fn term_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

fn main() -> Result<()> {
    let cli = mdr::cli::Cli::parse();

    match &cli.command {
        Some(mdr::cli::Commands::Serve { port, dir }) => {
            eprintln!("Server mode is not yet implemented.");
            eprintln!("Would serve markdown files from '{}' on port {}", dir, port);
            Ok(())
        }
        None => {
            match &cli.file {
                Some(path) => {
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", path, e))?;
                    let tw = term_width();
                    let rendered = mdr::render::terminal::render_with_width(&content, tw)?;
                    let links = mdr::render::terminal::extract_links(&rendered);
                    let config = mdr::tui::app::PagerConfig {
                        show_line_numbers: cli.line_numbers,
                        from_picker: false,
                        file_path: path.clone(),
                        links,
                    };
                    mdr::tui::app::run_with(rendered, config);
                    Ok(())
                }
                None => loop {
                    let file_path = match pick_markdown_file() {
                        Some(p) => p,
                        None => break Ok(()),
                    };

                    let content = match std::fs::read_to_string(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error reading '{}': {}", file_path, e);
                            continue;
                        }
                    };

                    let tw = term_width();
                    let rendered = match mdr::render::terminal::render_with_width(&content, tw) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Error rendering '{}': {}", file_path, e);
                            continue;
                        }
                    };

                    let links = mdr::render::terminal::extract_links(&rendered);
                    let picker_config = mdr::tui::app::PagerConfig {
                        show_line_numbers: cli.line_numbers,
                        from_picker: true,
                        file_path: file_path.clone(),
                        links,
                    };

                    match mdr::tui::app::run_with(rendered, picker_config) {
                        mdr::tui::app::PagerExit::GoBack => continue,
                        mdr::tui::app::PagerExit::Quit => break Ok(()),
                    }
                },
            }
        }
    }
}

// ── file info ────────────────────────────────────────────────────────

struct FileInfo {
    path: String,
    modified: String,
}

/// Format a `SystemTime` as `DD Mon YYYY  HH:MM` (Chinese-style).
fn fmt_time(t: SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let total_days = secs / 86400;
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;

    let mut y = 1970i64;
    let mut rem = total_days;
    loop {
        let days_in = if is_leap(y) { 366 } else { 365 };
        if rem < days_in {
            break;
        }
        rem -= days_in;
        y += 1;
    }

    let month_days: &[u64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut mo = 0u64;
    let mut d = rem;
    while mo < 12 && d >= month_days[mo as usize] {
        d -= month_days[mo as usize];
        mo += 1;
    }

    format!(
        "{} {}月 {}  {:02}:{:02}",
        d + 1,
        mo + 1,
        y,
        h,
        m,
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Collect markdown files under `dir` (max depth 3) with metadata.
fn collect_files(dir: &str) -> Vec<FileInfo> {
    let mut files: Vec<FileInfo> = walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
        })
        .map(|e| {
            let modified = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(fmt_time)
                .unwrap_or_default();
            FileInfo {
                path: e.path().to_string_lossy().to_string(),
                modified,
            }
        })
        .collect();

    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

// ── TUI file picker ─────────────────────────────────────────────────

fn pick_markdown_file() -> Option<String> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::ExecutableCommand;
    use ratatui::backend::CrosstermBackend;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;

    let all_files = collect_files(".");
    if all_files.is_empty() {
        eprintln!("No markdown files found.");
        return None;
    }

    enable_raw_mode().ok()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen).ok()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).ok()?;

    let mut filter = String::new();
    let mut selected = 0usize;
    let mut page = 0usize;
    const CARD_ROWS: u16 = 3; // name row + date row + gap

    let pick_result: Option<String> = 'outer: loop {
        // Filter
        let filtered: Vec<&FileInfo> = if filter.is_empty() {
            all_files.iter().collect()
        } else {
            let q = filter.to_lowercase();
            all_files
                .iter()
                .filter(|f| f.path.to_lowercase().contains(&q))
                .collect()
        };

        // ── render (uses a closure so we capture frame info before events) ──
        let render_result: Option<(usize, usize)> = {
            // We'll render and capture page_size / total_pages back.
            let mut result = None;

            let _ = terminal.draw(|frame| {
                let area = frame.area();
                let header_h: u16 = 5; // 1 blank + title + blank + info + blank
                let status_h: u16 = 2;
                let list_h = area.height.saturating_sub(header_h + status_h);
                let cards_per_page = (list_h / CARD_ROWS).max(1) as usize;

                let [header_area, list_area, status_area] =
                    Layout::vertical([
                        Constraint::Length(header_h),
                        Constraint::Min(1),
                        Constraint::Length(status_h),
                    ])
                    .areas(area);

                let total_filtered = filtered.len();
                let total_pages = total_filtered.div_ceil(cards_per_page).max(1);
                result = Some((cards_per_page, total_pages));

                // ── header: MDR + info ──
                let searching = !filter.is_empty();
                let logo_line = Line::from(Span::styled(
                    " MDR ",
                    Style::default()
                        .bg(Color::Rgb(0, 100, 200))
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                frame.render_widget(
                    Paragraph::new(logo_line),
                    Rect::new(header_area.x + 3, header_area.y + 1, header_area.width, 1),
                );

                // Row 3: document count + page info
                let page_str = if total_pages > 1 {
                    format!(" · page {}", page + 1)
                } else {
                    String::new()
                };
                let info_text = if searching {
                    format!("   {} of {} documents{}", total_filtered, all_files.len(), page_str)
                } else {
                    format!("   {} documents{}", total_filtered, page_str)
                };
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        info_text,
                        Style::default()
                            .fg(Color::Rgb(140, 140, 155))
                            .add_modifier(Modifier::DIM),
                    ))),
                    Rect::new(header_area.x, header_area.y + 3, header_area.width, 1),
                );

                // ── card list ──
                let card_w = list_area.width.saturating_sub(4).max(20) as usize;
                let card_x = list_area.x + 2;
                let theme = Color::Rgb(0, 100, 200);

                for ci in 0..cards_per_page {
                    let idx = page * cards_per_page + ci;
                    if idx >= filtered.len() {
                        break;
                    }
                    let entry = filtered[idx];
                    let is_selected = ci == selected;
                    let card_y = list_area.y + (ci as u16) * CARD_ROWS;

                    // Strip "./" prefix from top-level files
                    let display_path = entry.path.strip_prefix("./").unwrap_or(&entry.path);

                    let display_name = if display_path.len() > card_w.saturating_sub(2) {
                        format!(
                            "…{}",
                            &display_path[display_path.len().saturating_sub(card_w.saturating_sub(3))..]
                        )
                    } else {
                        display_path.to_string()
                    };

                    // ── Row 1: filename (with │ if selected) ──
                    let name_style = if is_selected {
                        Style::default().fg(theme).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let date_style = if is_selected {
                        Style::default().fg(Color::Rgb(100, 160, 220)).add_modifier(Modifier::DIM)
                    } else {
                        Style::default().fg(Color::Rgb(120, 120, 135)).add_modifier(Modifier::DIM)
                    };

                    let bar_prefix = if is_selected { "│" } else { " " };
                    let indent = "";

                    frame.render_widget(
                        Paragraph::new(Line::from(vec![
                            Span::styled(bar_prefix, if is_selected { Style::default().fg(theme) } else { Style::default().fg(Color::Rgb(80, 80, 90)) }),
                            Span::styled(format!("{} {}", indent, display_name), name_style),
                        ])),
                        Rect::new(card_x.saturating_sub(1), card_y, (card_w + 2) as u16, 1),
                    );

                    // ── Row 2: modified time ──
                    frame.render_widget(
                        Paragraph::new(Line::from(vec![
                            Span::styled(bar_prefix, if is_selected { Style::default().fg(theme) } else { Style::default().fg(Color::Rgb(80, 80, 90)) }),
                            Span::styled(format!("{} {}", indent, entry.modified), date_style),
                        ])),
                        Rect::new(card_x.saturating_sub(1), card_y + 1, (card_w + 2) as u16, 1),
                    );

                    // ── Row 3: gap (blank line between cards) ──
                    // nothing to render, just spacing
                }

                // ── bottom bar: shortcuts + search info ──
                let mut items: Vec<String> = Vec::new();

                if searching {
                    items.push(format!("FIND: {}", filter));
                }

                items.push("→ open".to_string());
                items.push("← back".to_string());
                items.push("q quit".to_string());

                let mut spans: Vec<Span<'static>> = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw("  ·  "));
                    }
                    if i == 0 && searching {
                        spans.push(Span::styled(
                            item.clone(),
                            Style::default().fg(Color::Rgb(200, 180, 80)).add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        spans.push(Span::styled(
                            item.clone(),
                            Style::default().fg(Color::Rgb(140, 140, 155)),
                        ));
                    }
                }

                frame.render_widget(
                    Paragraph::new(Line::from(spans)),
                    status_area,
                );
            });

            result
        };

        let (cards_per_page, total_pages) = render_result.unwrap_or((1, 1));

        if page >= total_pages {
            page = total_pages.saturating_sub(1);
        }
        let page_start = page * cards_per_page;
        let page_count = cards_per_page.min(filtered.len().saturating_sub(page_start));

        if selected >= page_count && page_count > 0 {
            selected = page_count - 1;
        } else if page_count == 0 {
            selected = 0;
        }



        // ── event handling ──
        if let Event::Key(key) = event::read().ok()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Left => break 'outer None,
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    } else if page > 0 {
                        page -= 1;
                        selected = cards_per_page.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < page_count {
                        selected += 1;
                    } else if page + 1 < total_pages {
                        page += 1;
                        selected = 0;
                    }
                }
                KeyCode::PageUp => {
                    if page > 0 {
                        page -= 1;
                        selected = 0;
                    }
                }
                KeyCode::PageDown => {
                    if page + 1 < total_pages {
                        page += 1;
                        selected = 0;
                    }
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    page = 0;
                    selected = 0;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    page = total_pages.saturating_sub(1);
                    if page == total_pages.saturating_sub(1) {
                        selected = page_count.saturating_sub(1);
                    } else {
                        selected = 0;
                    }
                }
                KeyCode::Enter | KeyCode::Right => {
                    let actual_idx = page_start + selected;
                    if actual_idx < filtered.len() {
                        break 'outer Some(filtered[actual_idx].path.clone());
                    }
                }
                KeyCode::Backspace => {
                    filter.pop();
                    selected = 0;
                    page = 0;
                }
                KeyCode::Char('/') if filter.is_empty() => {
                    // Just enter search mode, don't add / to query
                    // (typing any other char also enters search mode with that char)
                }
                KeyCode::Char(ch) => {
                    filter.push(ch);
                    selected = 0;
                    page = 0;
                }
                _ => {}
            }
        }
    };

    disable_raw_mode().ok()?;
    terminal.backend_mut().execute(LeaveAlternateScreen).ok()?;
    pick_result
}
