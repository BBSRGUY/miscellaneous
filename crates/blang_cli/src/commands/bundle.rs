//! Bundle command implementation

use crate::compiler::{compile_file, CompileOptions};
use crate::error::CliError;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// Run the bundle command
pub fn run(input: PathBuf, out_dir: PathBuf, release: bool) -> Result<()> {
    // Check input file exists
    if !input.exists() {
        return Err(CliError::ReadSource {
            path: input.clone(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        }
        .into());
    }

    info!(
        "Bundling {} to {} (release={})",
        input.display(),
        out_dir.display(),
        release
    );

    // Create output directory
    fs::create_dir_all(&out_dir).map_err(|e| CliError::CreateDirectory {
        path: out_dir.clone(),
        source: e,
    })?;

    // Compile to WASM
    let wasm_path = out_dir.join("app.wasm");
    let opt_level = if release { 2 } else { 0 };

    compile_file(
        &input,
        &wasm_path,
        CompileOptions {
            emit_ir: false,
            opt_level,
        },
    )?;

    // Copy runtime.js
    let runtime_js = include_str!("../../../../runtime/runtime.js");
    let runtime_path = out_dir.join("runtime.js");
    fs::write(&runtime_path, runtime_js).map_err(|e| CliError::WriteOutput {
        path: runtime_path.clone(),
        source: e,
    })?;

    // Generate HTML bootstrap
    let html = generate_html(&input);
    let html_path = out_dir.join("index.html");
    fs::write(&html_path, html).map_err(|e| CliError::WriteOutput {
        path: html_path.clone(),
        source: e,
    })?;

    // Copy any additional assets (if they exist)
    copy_assets(&input, &out_dir)?;

    println!("✓ Bundle created successfully!");
    println!("  Directory: {}", out_dir.display());
    println!("  Files:");
    println!("    - index.html");
    println!("    - app.wasm");
    println!("    - runtime.js");

    Ok(())
}

/// Generate HTML bootstrap file
fn generate_html(input: &PathBuf) -> String {
    let app_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Blang App");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
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
    <div id="app">
        <div class="loading">Loading Blang application...</div>
    </div>

    <script type="module">
        import {{ loadWasmModule, mountComponent }} from './runtime.js';

        async function main() {{
            try {{
                const response = await fetch('./app.wasm');
                const wasmBytes = await response.arrayBuffer();
                const instance = await loadWasmModule(new Uint8Array(wasmBytes));

                // Mount the main component if it exists
                const appElement = document.getElementById('app');
                appElement.innerHTML = '';

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
        app_name
    )
}

/// Copy additional assets from the input directory
fn copy_assets(input: &PathBuf, out_dir: &PathBuf) -> Result<()> {
    // Look for assets directory next to the input file
    if let Some(parent) = input.parent() {
        let assets_dir = parent.join("assets");
        if assets_dir.exists() && assets_dir.is_dir() {
            info!("Copying assets from {}", assets_dir.display());

            let dest_assets = out_dir.join("assets");
            fs::create_dir_all(&dest_assets).map_err(|e| CliError::CreateDirectory {
                path: dest_assets.clone(),
                source: e,
            })?;

            // Copy all files from assets directory
            copy_dir_recursive(&assets_dir, &dest_assets)?;

            println!("    - assets/");
        }
    }

    Ok(())
}

/// Recursively copy directory contents
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}
