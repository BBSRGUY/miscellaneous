//! Dev command implementation

use crate::dev_server;
use crate::error::CliError;
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Run the dev server command
pub fn run(dir: PathBuf, port: u16, open: bool) -> Result<()> {
    // Check directory exists
    if !dir.exists() {
        return Err(CliError::DirectoryNotFound(dir).into());
    }

    info!("Starting dev server for {} on port {}", dir.display(), port);

    // Open browser if requested
    if open {
        let url = format!("http://localhost:{}", port);
        info!("Opening browser at {}", url);

        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(&url).spawn().ok();

        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open").arg(&url).spawn().ok();

        #[cfg(target_os = "windows")]
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .ok();
    }

    // Start server (this blocks)
    tokio::runtime::Runtime::new()?.block_on(dev_server::start(dir, port))?;

    Ok(())
}
