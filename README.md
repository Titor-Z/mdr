# mdr — Markdown Renderer

> A modern, self-contained terminal Markdown renderer with a built-in pager.
> No `less`, no `more`, no external dependencies.

![demo](https://img.shields.io/badge/Rust-1.94%2B-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

---

## Features

### Terminal Viewer

- **Markdown rendering** — headings, paragraphs, code blocks (with syntax highlighting), lists, tables, task lists (☐/☑), blockquotes, horizontal rules, bold/italic/strikethrough, inline code, links, images
- **Built-in pager** — scroll with `↑↓`/`jk`, page with `PgUp`/`PgDn`/`b`/`f`, half-page with `u`/`d`
- **Search** — `/` to search, `n`/`N` to jump between matches
- **Help drawer** — `?` to toggle keyboard shortcuts panel
- **Line numbers** — `-l` or `L` to toggle
- **Mouse support** — scroll wheel, click & drag scrollbar, `Ctrl+m` to toggle mouse capture for text selection
- **Ctrl+click** — hold `Ctrl` and click on a link to open in browser

### File Picker (Home Screen)

```
  ████████ MDR ████████

   10 documents · page 1

  │ README.md
  │ 25 7月 2026  14:30

    notes/git命令集合.md
    17 3月 2026  18:52
  
  FIND: query  ·  → open  ·  ← back  ·  q quit
```

- Real-time search filtering
- Card-style layout with modification time
- Pagination (`PgUp`/`PgDn`, `gg`/`GG`)
- Chinese date format

### Tables

```
┌──────────┬──────────────────────────────────┬──────────┐
│ Name     │ Description                      │ Status   │
├──────────┼──────────────────────────────────┼──────────┤
│ Alice    │ A very long description that     │ Active   │
│          │ wraps across multiple lines      │          │
├──────────┼──────────────────────────────────┼──────────┤
│ Bob      │ Short                            │ Inactive │
└──────────┴──────────────────────────────────┴──────────┘
```

- Full grid borders (`┌─┬─┐│├─┼─┤└─┴─┘`)
- Auto column width distribution
- Word wrapping with vertical centering
- Multi-byte character support (CJK)

---

## Installation

### From source

```bash
git clone https://github.com/titor/mdr.git
cd mdr
cargo build --release
cp target/release/mdr /usr/local/bin/
```

### Requirements

- Rust 1.73+
- A terminal with true color support (for syntax highlighting)

---

## Usage

```bash
# Open a file directly
mdr README.md

# Open file picker (browse current directory)
mdr

# Show line numbers
mdr -l README.md

# HTTP server (coming soon)
mdr serve --port 8080
```

### Key Bindings

| Key | Action |
|-----|--------|
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `PgUp`/`b` | Page up |
| `PgDn`/`f` | Page down |
| `u`/`d` | Half page up/down |
| `g`/`G` | Top/bottom |
| `/` | Search |
| `n`/`N` | Next/previous match |
| `?` | Toggle help drawer |
| `L` | Toggle line numbers |
| `Ctrl+m` | Toggle mouse capture |
| `Ctrl+click` | Open link in browser |
| `q`/`Esc` | Quit / go back |

---

## Comparison with glow

| Feature | glow | mdr |
|---------|------|-----|
| Language | Go | Rust |
| External pager | Requires `less`/`more` | Built-in ratatui pager |
| Mouse support | ❌ | ✅ Scroll, click, drag |
| Table rendering | Plain text | ✅ Grid borders + wrapping |
| File picker | Simple list | Card layout + pagination |
| Binary size | ~15 MB | ~5 MB (release) |

---

## Project Status

**Current version**: v0.2.0

- ✅ Markdown rendering with syntax highlighting
- ✅ Built-in TUI pager
- ✅ File picker with search and pagination
- ✅ Table rendering with borders and wrapping
- ✅ Paragraph word wrapping
- ✅ Mouse interaction
- ✅ Search with match highlighting
- 🚧 HTTP server (`mdr serve`) — in development
- 📝 Config file support — planned

---

## License

MIT
