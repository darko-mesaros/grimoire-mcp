use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::response::Html;
use axum::routing::get;
use grimoire_mcp::{Pattern, load_all_patterns};
use pulldown_cmark::{Options, Parser, html};

const CSS: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #fafafa;
    margin: 0;
    padding: 20px;
}
.container {
    max-width: 800px;
    margin: 0 auto;
    background: #fff;
    padding: 30px 40px;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}
h1 {
    color: #2c3e50;
    border-bottom: 2px solid #3498db;
    padding-bottom: 10px;
}
h2 {
    color: #34495e;
}
a {
    color: #3498db;
    text-decoration: none;
}
a:hover {
    text-decoration: underline;
}
.pattern-list {
    list-style: none;
    padding: 0;
}
.pattern-list li {
    padding: 12px 16px;
    margin-bottom: 8px;
    background: #f8f9fa;
    border-radius: 6px;
    border-left: 4px solid #3498db;
}
.pattern-list li:hover {
    background: #eef2f7;
}
.pattern-name {
    font-weight: 600;
    font-size: 1.05em;
}
.pattern-meta {
    font-size: 0.85em;
    color: #666;
    margin-top: 4px;
}
.tag {
    display: inline-block;
    background: #e8f4fd;
    color: #2980b9;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 0.8em;
    margin-right: 4px;
}
dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 16px;
    background: #f8f9fa;
    padding: 16px;
    border-radius: 6px;
    margin-bottom: 24px;
}
dt {
    font-weight: 600;
    color: #555;
}
dd {
    margin: 0;
}
.back-link {
    display: inline-block;
    margin-bottom: 20px;
    font-size: 0.95em;
}
pre {
    background: #282c34;
    color: #abb2bf;
    padding: 16px;
    border-radius: 6px;
    overflow-x: auto;
}
code {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 0.9em;
}
p code, li code {
    background: #f0f0f0;
    padding: 2px 6px;
    border-radius: 3px;
}
"#;

struct AppState {
    patterns: Vec<Pattern>,
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut items = String::new();
    for pattern in &state.patterns {
        let name = &pattern.metadata.pattern;
        let category = &pattern.metadata.category;
        let tags_html: String = pattern
            .metadata
            .tags
            .iter()
            .map(|t| format!(r#"<span class="tag">{}</span>"#, html_escape(t)))
            .collect::<Vec<_>>()
            .join("");

        items.push_str(&format!(
            r#"<li>
                <div class="pattern-name"><a href="/pattern/{encoded_name}">{name}</a></div>
                <div class="pattern-meta">Category: {category} {tags_html}</div>
            </li>"#,
            encoded_name = urlencoding(name),
            name = html_escape(name),
            category = html_escape(category),
            tags_html = tags_html,
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Code Grimoire - Patterns</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        <h1>Code Grimoire</h1>
        <p>{count} patterns available</p>
        <ul class="pattern-list">
            {items}
        </ul>
    </div>
</body>
</html>"#,
        css = CSS,
        count = state.patterns.len(),
        items = items,
    );

    Html(html)
}

async fn pattern_detail(
    AxumPath(name): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let pattern = state
        .patterns
        .iter()
        .find(|p| p.metadata.pattern == name);

    let Some(pattern) = pattern else {
        return Html(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Pattern Not Found</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        <a href="/" class="back-link">&larr; Back to all patterns</a>
        <h1>Pattern Not Found</h1>
        <p>No pattern named "{name}" was found.</p>
    </div>
</body>
</html>"#,
            css = CSS,
            name = html_escape(&name),
        ));
    };

    // Render markdown to HTML.
    // NOTE: Raw HTML in markdown passes through unsanitized. This is acceptable
    // because this server binds to localhost only and patterns are author-controlled
    // trusted content. If the server is ever exposed to a network or patterns become
    // user-contributed, add an HTML sanitizer (e.g. ammonia).
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&pattern.content, options);
    let mut rendered_content = String::new();
    html::push_html(&mut rendered_content, parser);

    // Build metadata section
    let mut meta_html = String::from("<dl>");
    meta_html.push_str(&format!(
        "<dt>Category</dt><dd>{}</dd>",
        html_escape(&pattern.metadata.category)
    ));
    if let Some(ref fw) = pattern.metadata.framework {
        meta_html.push_str(&format!(
            "<dt>Framework</dt><dd>{}</dd>",
            html_escape(fw)
        ));
    }
    if !pattern.metadata.projects.is_empty() {
        meta_html.push_str(&format!(
            "<dt>Projects</dt><dd>{}</dd>",
            html_escape(&pattern.metadata.projects.join(", "))
        ));
    }
    if !pattern.metadata.tags.is_empty() {
        let tags: String = pattern
            .metadata
            .tags
            .iter()
            .map(|t| format!(r#"<span class="tag">{}</span>"#, html_escape(t)))
            .collect::<Vec<_>>()
            .join(" ");
        meta_html.push_str(&format!("<dt>Tags</dt><dd>{}</dd>", tags));
    }
    meta_html.push_str("</dl>");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title} - Code Grimoire</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        <a href="/" class="back-link">&larr; Back to all patterns</a>
        <h1>{title}</h1>
        {meta_html}
        <div class="content">
            {rendered_content}
        </div>
    </div>
</body>
</html>"#,
        css = CSS,
        title = html_escape(&pattern.metadata.pattern),
        meta_html = meta_html,
        rendered_content = rendered_content,
    );

    Html(html)
}

/// Simple HTML escaping for dynamic content
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Simple URL encoding for pattern names.
/// NOTE: This relies on validate_pattern_name restricting names to [a-zA-Z0-9 _-],
/// so we only need to handle spaces and underscores as special bytes. Pattern names
/// loaded from disk could theoretically contain other characters, but in practice
/// they are all authored through the MCP create_pattern tool which enforces the
/// character restriction.
fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Validate PATTERNS_DIR before starting the server
    let patterns_dir = match std::env::var("PATTERNS_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Error: PATTERNS_DIR environment variable is not set.");
            eprintln!("Set it to the directory containing your pattern .md files.");
            std::process::exit(1);
        }
    };

    if !Path::new(&patterns_dir).is_dir() {
        eprintln!(
            "Error: PATTERNS_DIR '{}' does not exist or is not a directory.",
            patterns_dir
        );
        std::process::exit(1);
    }

    // Patterns are loaded once at startup. There is no hot-reload mechanism;
    // file changes require restarting the server.
    // Safety: load_all_patterns() reads PATTERNS_DIR internally via .expect(),
    // but we have already validated the env var above so it will not panic.
    let patterns = load_all_patterns();

    let state = Arc::new(AppState { patterns });

    let app = Router::new()
        .route("/", get(index))
        .route("/pattern/{name}", get(pattern_detail))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("127.0.0.1:{}", port);
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
