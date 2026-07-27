use std::io;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseEventKind,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use unicode_width::UnicodeWidthStr;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Terminal;



/// Signal returned by the pager when the user quits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerExit {
    /// Quit the program entirely.
    Quit,
    /// Return to the file picker (called from `mdr` with no args).
    GoBack,
}

/// Options for the TUI pager.
#[derive(Default)]
pub struct PagerConfig {
    pub show_line_numbers: bool,
    /// When true, pressing `q` / `Esc` returns [`PagerExit::GoBack`]
    /// instead of [`PagerExit::Quit`].
    pub from_picker: bool,
    /// File path to show in the status bar.
    pub file_path: String,
    /// Link positions in rendered content (for Ctrl+click).
    pub links: Vec<crate::render::LinkInfo>,
}

/// Run the TUI pager with the given content lines.
pub fn run(content: Vec<Line<'static>>) -> PagerExit {
    run_with(content, PagerConfig::default())
}

/// Run the TUI pager with custom config.
pub fn run_with(content: Vec<Line<'static>>, config: PagerConfig) -> PagerExit {
    enable_raw_mode().ok();
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).ok();
    stdout.execute(EnableMouseCapture).ok();
    let terminal = Terminal::new(CrosstermBackend::new(stdout)).ok();

    let mut terminal = match terminal {
        Some(t) => t,
        None => return PagerExit::Quit,
    };

    let mut app = App::new(content, config);
    let exit = app.run(&mut terminal);

    terminal.backend_mut().execute(DisableMouseCapture).ok();
    terminal.backend_mut().execute(LeaveAlternateScreen).ok();
    disable_raw_mode().ok();
    exit
}

// ── App State ────────────────────────────────────────────────────────

enum InputMode {
    Normal,
    Searching,
}

struct SearchState {
    query: String,
    /// Indices into `content` where the match was found.
    matches: Vec<usize>,
    /// Index into `matches` (0-based).
    current: usize,
}

/// Tracks an in-progress mouse-drag on the scrollbar.
struct DragState {
    /// The terminal row where the drag started (0-based).
    _start_row: u16,
    /// The scroll offset when the drag started.
    _start_scroll: usize,
}

struct App {
    content: Vec<Line<'static>>,
    /// Pre-computed plain text of each line (for search).
    line_texts: Vec<String>,
    scroll: usize,
    show_line_numbers: bool,

    input_mode: InputMode,
    search: Option<SearchState>,

    /// Cached from last render so mouse handler knows the scrollbar column.
    last_content_area: Rect,
    /// Non-None while the user is dragging the scrollbar thumb.
    drag: Option<DragState>,

    /// When true, `q`/`Esc` returns [`PagerExit::GoBack`].
    should_go_back: bool,
    /// Whether the help drawer is open.
    show_help: bool,
    /// Whether mouse capture is enabled (for toggling text selection).
    mouse_enabled: bool,
    /// File path for status bar display.
    file_path: String,
    /// Link positions in the rendered content.
    links: Vec<crate::render::LinkInfo>,
}

impl App {
    fn new(content: Vec<Line<'static>>, config: PagerConfig) -> Self {
        let line_texts: Vec<String> = content.iter().map(|l| l.to_string()).collect();
        Self {
            content,
            line_texts,
            scroll: 0,
            show_line_numbers: config.show_line_numbers,
            input_mode: InputMode::Normal,
            search: None,
            last_content_area: Rect::default(),
            drag: None,
            should_go_back: config.from_picker,
            show_help: false,
            mouse_enabled: true,
            file_path: config.file_path,
            links: config.links,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<impl io::Write>>) -> PagerExit {
        loop {
            let _ = terminal.draw(|frame| self.render(frame));

            let event = match event::read() {
                Ok(e) => e,
                Err(_) => return PagerExit::Quit,
            };
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match self.input_mode {
                        InputMode::Searching => self.handle_search_key(key),
                        InputMode::Normal => {
                            if let Some(exit) = self.handle_normal_key(key) {
                                return exit;
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => self.handle_mouse(mouse),
                _ => {}
            }
        }
    }

    // ── normal-mode key handling ────────────────────────────────────

    /// Returns `Some(exit)` if the user wants to quit / go back.
    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) -> Option<PagerExit> {
        let view_height = self.last_content_area.height.saturating_sub(1) as usize;

        match key.code {
            KeyCode::Left | KeyCode::Char('q') | KeyCode::Esc => {
                return Some(if self.should_go_back {
                    PagerExit::GoBack
                } else {
                    PagerExit::Quit
                });
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(PagerExit::Quit);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_scroll = self.content.len().saturating_sub(view_height);
                if self.scroll < max_scroll {
                    self.scroll += 1;
                }
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.scroll = self.scroll.saturating_sub(view_height);
            }
            KeyCode::PageDown | KeyCode::Char('f') => {
                let max_scroll = self.content.len().saturating_sub(view_height);
                self.scroll = (self.scroll + view_height).min(max_scroll);
            }
            KeyCode::Char('u') => {
                let half = view_height / 2;
                self.scroll = self.scroll.saturating_sub(half.max(1));
            }
            KeyCode::Char('d') => {
                let half = view_height / 2;
                let max_scroll = self.content.len().saturating_sub(view_height);
                self.scroll = (self.scroll + half.max(1)).min(max_scroll);
            }
            KeyCode::Home | KeyCode::Char('g') => self.scroll = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll = self.content.len().saturating_sub(view_height);
            }
            KeyCode::Char('/') => {
                let q = self.search.as_ref().map(|s| s.query.clone()).unwrap_or_default();
                let has_query = !q.is_empty();
                self.input_mode = InputMode::Searching;
                self.search = Some(SearchState {
                    query: q,
                    matches: vec![],
                    current: 0,
                });
                if has_query {
                    self.update_search_matches();
                }
            }
            KeyCode::Char('n') => self.next_match(),
            KeyCode::Char('N') => self.prev_match(),
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('L') => self.show_line_numbers = !self.show_line_numbers,
            KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mouse_enabled = !self.mouse_enabled;
            }
            _ => {}
        }
        None
    }

    // ── mouse handling ──────────────────────────────────────────────

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        // When mouse capture is off, skip all mouse events
        // (terminal handles text selection natively with Shift+click)
        if !self.mouse_enabled {
            return;
        }

        let view_height = self.last_content_area.height.saturating_sub(1) as usize;
        let max_scroll = self.content.len().saturating_sub(view_height);
        let scrollbar_x = self.last_content_area.right().saturating_sub(1);
        let sb_top = self.last_content_area.y;
        let sb_bot = self.last_content_area.bottom().saturating_sub(1);
        let sb_height = sb_bot.saturating_sub(sb_top).max(1) as usize;

        // Ctrl+click on content → open link
        if mouse.modifiers.contains(KeyModifiers::CONTROL) {
            if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                let row = mouse.row;
                let _col = mouse.column;
                let clicked_line = self.scroll + (row.saturating_sub(self.last_content_area.y)) as usize;
                // Find a link on this line
                for link in &self.links {
                    if link.line == clicked_line {
                        let url = link.url.clone();
                        std::thread::spawn(move || {
                            let _ = std::process::Command::new("open")
                                .arg(&url)
                                .spawn();
                        });
                        return;
                    }
                }
            }
            return;
        }

        match mouse.kind {
            // ── scroll wheel anywhere ──
            MouseEventKind::ScrollDown => {
                if self.scroll < max_scroll {
                    self.scroll = self.scroll.saturating_add(3).min(max_scroll);
                }
            }
            MouseEventKind::ScrollUp => {
                self.scroll = self.scroll.saturating_sub(3);
            }

            // ── left-click on scrollbar → jump ──
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let col = mouse.column;
                let row = mouse.row;

                if col == scrollbar_x && row >= sb_top && row <= sb_bot {
                    let ratio = (row - sb_top) as f64 / sb_height as f64;
                    let target = (ratio * max_scroll as f64).round() as usize;
                    self.scroll = target.min(max_scroll);
                    self.drag = Some(DragState {
                        _start_row: row,
                        _start_scroll: self.scroll,
                    });
                }
            }

            MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                if self.drag.is_some() {
                    let row = mouse.row;
                    if row >= sb_top && row <= sb_bot {
                        let ratio = (row - sb_top) as f64 / sb_height as f64;
                        let target = (ratio * max_scroll as f64).round() as usize;
                        self.scroll = target.min(max_scroll);
                    } else if row < sb_top {
                        self.scroll = 0;
                    } else {
                        self.scroll = max_scroll;
                    }
                }
            }

            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                self.drag = None;
            }

            _ => {}
        }
    }

    // ── search-mode key handling ────────────────────────────────────

    fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter | KeyCode::Down | KeyCode::Char('n') => {
                self.next_match();
            }
            KeyCode::Up | KeyCode::Char('p') => {
                self.prev_match();
            }
            KeyCode::Backspace => {
                if let Some(ref mut s) = self.search {
                    s.query.pop();
                }
                self.update_search_matches();
            }
            KeyCode::Char(ch) => {
                if let Some(ref mut s) = self.search {
                    s.query.push(ch);
                }
                self.update_search_matches();
            }
            _ => {}
        }
    }

    // ── search logic ────────────────────────────────────────────────

    fn update_search_matches(&mut self) {
        let search = self.search.as_mut().unwrap();
        if search.query.is_empty() {
            search.matches.clear();
            search.current = 0;
            return;
        }

        let q = search.query.to_lowercase();
        search.matches = self
            .line_texts
            .iter()
            .enumerate()
            .filter(|(_, text)| text.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        search.current = 0;

        if let Some(&first) = search.matches.first() {
            self.jump_to_line(first);
        }
    }

    fn next_match(&mut self) {
        let current = {
            let search = match self.search.as_mut() {
                Some(s) if !s.matches.is_empty() => s,
                _ => return,
            };
            if search.current + 1 < search.matches.len() {
                search.current += 1;
            } else {
                search.current = 0;
            }
            search.matches[search.current]
        };
        self.jump_to_line(current);
    }

    fn prev_match(&mut self) {
        let current = {
            let search = match self.search.as_mut() {
                Some(s) if !s.matches.is_empty() => s,
                _ => return,
            };
            if search.current > 0 {
                search.current -= 1;
            } else {
                search.current = search.matches.len().saturating_sub(1);
            }
            search.matches[search.current]
        };
        self.jump_to_line(current);
    }

    fn jump_to_line(&mut self, line: usize) {
        if line >= 3 {
            self.scroll = line.saturating_sub(3);
        } else {
            self.scroll = 0;
        }
    }

    // ── rendering ───────────────────────────────────────────────────

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let total = self.content.len();

        // Layout: main content area + status/search bar + help drawer (optional)
        let help_h: u16 = if self.show_help { 8 } else { 0 };
        let layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(help_h),
        ]);
        let [raw_content, status_area, help_area] = layout.areas(area);
        self.last_content_area = raw_content;

        // 1-column horizontal padding so content doesn't touch edges or scrollbar
        let content_area = Rect {
            x: raw_content.x + 1,
            y: raw_content.y,
            width: raw_content.width.saturating_sub(2),
            height: raw_content.height,
        };

        let view_height = raw_content.height.saturating_sub(1) as usize;
        let max_scroll = total.saturating_sub(view_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        // ── build visible lines with line numbers and search highlights ──
        let line_num_width = if self.show_line_numbers {
            total.to_string().len()
        } else {
            0
        };

        let highlight_style = Style::default()
            .bg(Color::Rgb(100, 30, 30))
            .add_modifier(Modifier::BOLD);
        let current_match_style = Style::default()
            .bg(Color::Rgb(180, 30, 30))
            .add_modifier(Modifier::BOLD);

        let visible_lines: Vec<Line> = self
            .content
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(view_height)
            .map(|(i, line)| {
                let is_current = self.search.as_ref().is_some_and(|s| {
                    s.matches.get(s.current) == Some(&i)
                });
                let is_other = self.search.as_ref().is_some_and(|s| {
                    s.matches.contains(&i) && !is_current
                });

                let mut spans: Vec<Span<'static>> = Vec::new();

                // Line number prefix
                if self.show_line_numbers {
                    let num_str = format!("{:>width$} ", i + 1, width = line_num_width);
                    spans.push(Span::styled(
                        num_str,
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ));
                }

                let line_style = if is_current {
                    current_match_style
                } else if is_other {
                    highlight_style
                } else {
                    Style::default()
                };

                for span in &line.spans {
                    let merged = span.style.patch(line_style);
                    spans.push(Span::styled(span.content.clone(), merged));
                }

                Line::from(spans)
            })
            .collect();

        // ── content area ──
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(Color::DarkGray));

        let mut scrollbar_state =
            ScrollbarState::new(max_scroll).position(self.scroll);

        let content_paragraph = Paragraph::new(visible_lines)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(content_paragraph, content_area);

        // Scrollbar at the terminal's rightmost edge, outside content padding
        let scrollbar_area = Rect {
            x: area.right().saturating_sub(1),
            y: content_area.y,
            width: 1,
            height: content_area.height,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

        // ── status / search bar (always at bottom) ──
        match self.input_mode {
            InputMode::Normal => {
                self.render_status_bar(frame, status_area, view_height, total);
            }
            InputMode::Searching => {
                self.render_search_bar(frame, status_area);
            }
        }

        // ── help drawer (slides up from below the status bar) ──
        if self.show_help {
            self.render_help(frame, help_area);
        }
    }

    fn render_status_bar(
        &self,
        frame: &mut ratatui::Frame,
        area: Rect,
        view_height: usize,
        total: usize,
    ) {
        let max_scroll = total.saturating_sub(view_height);
        let pos_pct = if total == 0 || max_scroll == 0 {
            100.0
        } else {
            (self.scroll as f64 / max_scroll as f64) * 100.0
        };

        // Build two-line status: path (left) + scroll% + ? Help (right)
        let path = if self.file_path.is_empty() {
            "<unknown>".to_string()
        } else {
            self.file_path.clone()
        };

        let right = format!("{:.0}%  ? Help", pos_pct);

        // Pad the middle so right side is right-aligned
        let avail = area.width.saturating_sub(7) as usize; // " MDR " + "  "
        let right_len = right.width();
        let path_max = avail.saturating_sub(right_len + 2);
        let display_path = if path.width() > path_max {
            // Truncate path to fit, prepend ellipsis
            let keep = path_max.saturating_sub(1);
            let truncated: String = path.chars().rev().take(keep).collect::<Vec<_>>().into_iter().rev().collect();
            format!("…{}", truncated)
        } else {
            path
        };

        let padding = avail.saturating_sub(display_path.width() + right_len);
        let status = format!("{}{}{}", display_path, " ".repeat(padding), right);

        let logo_style = Style::default()
            .bg(Color::Rgb(0, 100, 200))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .fg(Color::Rgb(180, 180, 190))
            .bg(Color::Rgb(30, 30, 40));

        // Use normal_style as paragraph fallback so unfilled cells
        // still get the dark background, making the bar span 100% width.
        let status_bar = Paragraph::new(Line::from(vec![
            Span::styled(" MDR ", logo_style),
            Span::styled(format!("  {}", status), normal_style),
        ]))
        .style(normal_style);
        frame.render_widget(status_bar, area);
    }

    fn render_help(&self, frame: &mut ratatui::Frame, area: Rect) {
        let bg = Style::default().bg(Color::Rgb(24, 24, 34));
        let key = Style::default()
            .fg(Color::Rgb(0, 160, 220))
            .add_modifier(Modifier::BOLD);
        let desc = Style::default().fg(Color::Rgb(160, 160, 170));

        let mouse_status = if self.mouse_enabled { "on" } else { "off" };

        let rows: Vec<Line> = [
            ("k/↑",    "up",            "g/home",    "go to top"),
            ("j/↓",    "down",          "G/end",     "go to bottom"),
            ("b/pgup", "page up",       "?",         "toggle help"),
            ("f/pgdn", "page down",     "q/esc",     "quit"),
            ("u",      "½ page up",     "/",         "search"),
            ("d",      "½ page down",   "n",         "next match"),
            ("L",      "line numbers",  "N",         "prev match"),
            ("C-m",    &format!("mouse {}", mouse_status), "C-click", "open link"),
        ]
        .iter()
        .map(|(k1, d1, k2, d2)| {
            Line::from(vec![
                Span::styled(format!("  {:<8}", k1), key),
                Span::styled(format!("{:<12}", d1), desc),
                Span::styled(format!("  {:<7}", k2), key),
                Span::styled(format!("{}", d2), desc),
            ])
        })
        .collect();

        frame.render_widget(Paragraph::new(ratatui::text::Text::from(rows)).style(bg), area);
    }

    fn render_search_bar(&self, frame: &mut ratatui::Frame, area: Rect) {
        let query = self
            .search
            .as_ref()
            .map(|s| s.query.as_str())
            .unwrap_or("");

        let match_info = self.search.as_ref().map_or(String::new(), |s| {
            if s.matches.is_empty() {
                if s.query.is_empty() {
                    String::new()
                } else {
                    "  (no matches)".to_string()
                }
            } else {
                format!("  {}/{}", s.current + 1, s.matches.len())
            }
        });

        let hint = if query.is_empty() {
            "  type to search".to_string()
        } else {
            format!("  {}{}", match_info,
                if self.search.as_ref().map_or(false, |s| !s.matches.is_empty()) {
                    ""  // match info already shown
                } else {
                    ""
                })
        };

        let text = format!("/{}{}", query, hint);

        let search_style = Style::default()
            .fg(Color::Rgb(0, 160, 230))
            .bg(Color::Rgb(30, 30, 40))
            .add_modifier(Modifier::BOLD);

        // Pad to fill full width so background spans entire bar
        let padding = area.width.saturating_sub(text.width() as u16).saturating_sub(1);
        let padded = format!("{} {}", text, " ".repeat(padding as usize));

        let search_bar = Paragraph::new(Line::from(Span::styled(padded, search_style)))
            .style(search_style);

        frame.render_widget(search_bar, area);

        let cursor_x = 1 + query.width() as u16;
        frame.set_cursor_position((area.x + cursor_x, area.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app(lines: &[&str]) -> App {
        let content: Vec<Line<'static>> = lines
            .iter()
            .map(|s| Line::from(Span::raw(s.to_string())))
            .collect();
        App::new(content, PagerConfig::default())
    }

    #[test]
    fn test_search_finds_matches() {
        let mut app = make_app(&["hello world", "foo bar", "Hello again", "no match here"]);

        app.input_mode = InputMode::Searching;
        app.search = Some(SearchState {
            query: String::new(),
            matches: vec![],
            current: 0,
        });
        app.search.as_mut().unwrap().query = "hello".to_string();
        app.update_search_matches();

        assert_eq!(app.search.as_ref().unwrap().matches, vec![0, 2]);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut app = make_app(&["Hello World", "goodbye world", "HELLO AGAIN"]);

        app.input_mode = InputMode::Searching;
        app.search = Some(SearchState {
            query: String::new(),
            matches: vec![],
            current: 0,
        });
        app.search.as_mut().unwrap().query = "hello".to_string();
        app.update_search_matches();

        assert_eq!(app.search.as_ref().unwrap().matches, vec![0, 2]);
    }

    #[test]
    fn test_search_no_matches() {
        let mut app = make_app(&["alpha", "beta", "gamma"]);

        app.input_mode = InputMode::Searching;
        app.search = Some(SearchState {
            query: String::new(),
            matches: vec![],
            current: 0,
        });
        app.search.as_mut().unwrap().query = "delta".to_string();
        app.update_search_matches();

        assert!(app.search.as_ref().unwrap().matches.is_empty());
    }

    #[test]
    fn test_search_navigation() {
        let mut app = make_app(&[
            "aaa match bbb",
            "ccc ddd",
            "eee match fff",
            "ggg hhh",
            "iii match jjj",
        ]);

        app.search = Some(SearchState {
            query: "match".to_string(),
            matches: vec![0, 2, 4],
            current: 0,
        });

        assert_eq!(app.search.as_ref().unwrap().current, 0);

        app.next_match();
        assert_eq!(app.search.as_ref().unwrap().current, 1);

        app.next_match();
        assert_eq!(app.search.as_ref().unwrap().current, 2);

        app.next_match();
        assert_eq!(app.search.as_ref().unwrap().current, 0);

        app.prev_match();
        assert_eq!(app.search.as_ref().unwrap().current, 2);
    }

    #[test]
    fn test_search_empty_query() {
        let mut app = make_app(&["hello", "world"]);

        app.input_mode = InputMode::Searching;
        app.search = Some(SearchState {
            query: String::new(),
            matches: vec![],
            current: 0,
        });
        app.update_search_matches();
        assert!(app.search.as_ref().unwrap().matches.is_empty());
    }
}
