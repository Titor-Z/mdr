use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path as AxumPath, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

struct AppState {
    env: Environment<'static>,
    docs: Vec<DocInfo>,
    serve_dir: PathBuf,
    theme: AtomicU16,
}

/// Document metadata for templates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocInfo {
    pub path: String,
    pub modified: String,
}

/// Start the HTTP server.
pub async fn start(port: u16, dir: String) {
    let dir = std::fs::canonicalize(&dir).unwrap_or_else(|_| Path::new(&dir).to_path_buf());
    eprintln!("Serving markdown files from: {}", dir.display());

    // Build template environment
    let mut env = Environment::new();
    env.add_template_owned("base.html", include_str!("templates/base.html"))
        .expect("base.html");
    env.add_template_owned("index.html", include_str!("templates/index.html"))
        .expect("index.html");
    env.add_template_owned("document.html", include_str!("templates/document.html"))
        .expect("document.html");

    let docs = scan_docs(&dir);

    // 开发模式：从文件系统加载模板（改样式后重启即可，无需编译）
    if std::env::var("MDR_DEV").is_ok() {
        let template_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/server/templates");
        let base = std::fs::read_to_string(format!("{}/base.html", template_dir)).unwrap_or_default();
        let index = std::fs::read_to_string(format!("{}/index.html", template_dir)).unwrap_or_default();
        let doc = std::fs::read_to_string(format!("{}/document.html", template_dir)).unwrap_or_default();
        env.add_template_owned("base.html", base).expect("base.html");
        env.add_template_owned("index.html", index).expect("index.html");
        env.add_template_owned("document.html", doc).expect("document.html");
        eprintln!("  [dev] 从文件系统加载模板");
    }

    let state = Arc::new(AppState {
        env,
        docs,
        serve_dir: dir,
        theme: AtomicU16::new(0),
    });

    let assets_css = include_str!("templates/style.css");
    let assets_js = include_str!("templates/script.js");

    let app = Router::new()
        .route("/", get(index_page))
        .route("/docs/{*path}", get(doc_page))
        .route("/assets/style.css", get(move || async move {
            ([(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")], assets_css)
        }))
        .route("/assets/script.js", get(move || async move {
            ([(axum::http::header::CONTENT_TYPE, "application/javascript; charset=utf-8")], assets_js)
        }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    eprintln!("Server running at http://localhost:{}", port);
    axum::serve(listener, app).await.expect("Server error");
}

// ── handlers ────────────────────────────────────────────────────────

async fn index_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let t = state.env.get_template("index.html").unwrap();
    let theme_str = if state.theme.load(Ordering::Relaxed) == 0 {
        "dark"
    } else {
        "light"
    };
    let html = t
        .render(minijinja::context! {
            docs => &state.docs,
            theme => theme_str,
        })
        .unwrap();
    Html(html)
}

async fn doc_page(
    State(state): State<Arc<AppState>>,
    AxumPath(path): AxumPath<String>,
) -> Html<String> {
    let file_path = state.serve_dir.join(&path);

    if !file_path.exists() || !file_path.is_file() {
        return Html(format!("404 Not Found: {}", path));
    }

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => return Html(format!("Error reading file: {}", e)),
    };

    let html_content = crate::server::render::markdown_to_html(&content);
    let toc = crate::server::render::extract_toc(&content);
    let title = path.rsplit('/').next().unwrap_or(&path).to_string();

    let t = state.env.get_template("document.html").unwrap();
    let theme_str = if state.theme.load(Ordering::Relaxed) == 0 {
        "dark"
    } else {
        "light"
    };
    let html = t
        .render(minijinja::context! {
            title => title,
            content => html_content,
            toc => toc,
            all_docs => &state.docs,
            current_path => path,
            theme => theme_str,
        })
        .unwrap();
    Html(html)
}

/// Scan a directory for .md files recursively (max depth 5).
fn scan_docs(dir: &Path) -> Vec<DocInfo> {
    let mut docs: Vec<DocInfo> = WalkDir::new(dir)
        .max_depth(5)
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
            // Compute relative path
            let abs_path = e.path();
            let rel = abs_path
                .strip_prefix(dir)
                .unwrap_or(abs_path)
                .to_string_lossy()
                .to_string();

            let modified = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(fmt_time)
                .unwrap_or_default();

            DocInfo {
                path: rel,
                modified,
            }
        })
        .collect();

    docs.sort_by(|a, b| a.path.cmp(&b.path));
    docs
}

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
