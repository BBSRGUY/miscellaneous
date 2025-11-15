//! Development server with hot reload

use crate::compiler::{compile_file, CompileOptions};
use crate::error::CliError;
use anyhow::Result;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::{error, info, warn};

/// Shared state for the dev server
#[derive(Clone)]
struct DevServerState {
    project_dir: PathBuf,
    reload_tx: broadcast::Sender<()>,
    compile_status: Arc<Mutex<CompileStatus>>,
}

#[derive(Clone)]
struct CompileStatus {
    last_error: Option<String>,
}

/// Start the development server
pub async fn start(project_dir: PathBuf, port: u16) -> Result<()> {
    info!("Starting dev server for {}", project_dir.display());

    // Create broadcast channel for reload signals
    let (reload_tx, _reload_rx) = broadcast::channel::<()>(100);

    // Create shared state
    let state = DevServerState {
        project_dir: project_dir.clone(),
        reload_tx: reload_tx.clone(),
        compile_status: Arc::new(Mutex::new(CompileStatus { last_error: None })),
    };

    // Initial compilation
    compile_project(&project_dir, &state)?;

    // Set up file watcher
    setup_file_watcher(project_dir.clone(), state.clone())?;

    // Build router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/__reload", get(reload_endpoint))
        .nest_service("/dist", ServeDir::new(project_dir.join("dist")))
        .nest_service("/assets", ServeDir::new(project_dir.join("assets")))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Dev server listening on http://{}", addr);
    println!("\n🚀 Blang dev server running at http://{}", addr);
    println!("   Press Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the index page with hot reload script
async fn serve_index(State(state): State<DevServerState>) -> impl IntoResponse {
    let html = generate_dev_html(&state);
    Html(html)
}

/// Server-sent events endpoint for hot reload
async fn reload_endpoint(State(state): State<DevServerState>) -> Response {
    let mut rx = state.reload_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(_) => {
                    yield Ok::<_, std::convert::Infallible>(
                        format!("data: reload\n\n")
                    );
                }
                Err(_) => break,
            }
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}

/// Generate HTML for dev server with hot reload script
fn generate_dev_html(state: &DevServerState) -> String {
    let error_html = if let Ok(status) = state.compile_status.lock() {
        if let Some(ref err) = status.last_error {
            format!(
                r#"<div style="position: fixed; top: 0; left: 0; right: 0; background: #ff4444; color: white; padding: 20px; z-index: 10000;">
                    <h3>Compilation Error</h3>
                    <pre style="background: rgba(0,0,0,0.2); padding: 10px; overflow-x: auto;">{}</pre>
                </div>"#,
                html_escape(err)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blang Dev Server</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
        #app {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .loading {{
            text-align: center;
            color: #666;
        }}
    </style>
</head>
<body>
    {}
    <div id="app">
        <div class="loading">Loading Blang application...</div>
    </div>

    <script type="module">
        // Hot reload support
        const eventSource = new EventSource('/__reload');
        eventSource.onmessage = (event) => {{
            if (event.data === 'reload') {{
                console.log('Reloading due to file changes...');
                window.location.reload();
            }}
        }};

        eventSource.onerror = () => {{
            console.warn('Hot reload connection lost');
        }};

        // Load the application
        async function main() {{
            try {{
                const response = await fetch('/dist/app.wasm');
                const wasmBytes = await response.arrayBuffer();

                const imports = {{
                    env: {{
                        println: (ptr, len) => {{
                            console.log('WASM output');
                        }}
                    }}
                }};

                const result = await WebAssembly.instantiate(wasmBytes, imports);
                const instance = result.instance;

                document.getElementById('app').innerHTML = '<h1>Blang App Running!</h1>';
                console.log('Blang app loaded successfully!');
                console.log('WASM exports:', Object.keys(instance.exports));

                // Call main if it exists
                if (instance.exports.main) {{
                    instance.exports.main();
                }}
            }} catch (error) {{
                document.getElementById('app').innerHTML = `
                    <div style="color: red;">
                        <h2>Error loading application</h2>
                        <pre>${{error.message}}</pre>
                    </div>
                `;
                console.error('Failed to load Blang app:', error);
            }}
        }}

        main();
    </script>
</body>
</html>"#,
        error_html
    )
}

/// Compile the project
fn compile_project(project_dir: &Path, state: &DevServerState) -> Result<()> {
    // Look for main.blang or src/main.blang
    let input_paths = vec![
        project_dir.join("main.blang"),
        project_dir.join("src/main.blang"),
        project_dir.join("src/lib.blang"),
    ];

    let input = input_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| {
            CliError::DirectoryNotFound(project_dir.to_path_buf())
        })?;

    let dist_dir = project_dir.join("dist");
    fs::create_dir_all(&dist_dir).map_err(|e| CliError::CreateDirectory {
        path: dist_dir.clone(),
        source: e,
    })?;

    let output = dist_dir.join("app.wasm");

    info!("Compiling {} -> {}", input.display(), output.display());

    match compile_file(
        input,
        &output,
        CompileOptions {
            emit_ir: false,
            opt_level: 0,
        },
    ) {
        Ok(_) => {
            info!("✓ Compilation successful");
            if let Ok(mut status) = state.compile_status.lock() {
                status.last_error = None;
            }
            Ok(())
        }
        Err(e) => {
            error!("✗ Compilation failed: {}", e);
            if let Ok(mut status) = state.compile_status.lock() {
                status.last_error = Some(e.to_string());
            }
            Err(e.into())
        }
    }
}

/// Set up file watcher for hot reload
fn setup_file_watcher(project_dir: PathBuf, state: DevServerState) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;

    watcher.watch(&project_dir, RecursiveMode::Recursive)?;

    // Spawn watcher thread
    std::thread::spawn(move || {
        let mut _watcher = watcher; // Keep watcher alive

        for res in rx {
            match res {
                Ok(event) => {
                    if should_trigger_reload(&event) {
                        info!("File change detected, recompiling...");

                        // Recompile
                        if let Err(e) = compile_project(&project_dir, &state) {
                            warn!("Compilation failed: {}", e);
                        }

                        // Trigger reload
                        let _ = state.reload_tx.send(());
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    });

    Ok(())
}

/// Check if an event should trigger a reload
fn should_trigger_reload(event: &Event) -> bool {
    use notify::EventKind;

    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            // Check if it's a Blang source file
            event.paths.iter().any(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "blang")
                    .unwrap_or(false)
            })
        }
        _ => false,
    }
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
