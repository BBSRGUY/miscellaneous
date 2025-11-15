//! Blang CLI - Command-line interface for the Blang programming language

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod compiler;
mod dev_server;
mod error;

use commands::{bundle, compile, dev};

/// Blang compiler and development tools
#[derive(Parser)]
#[command(name = "blang")]
#[command(about = "Blang programming language compiler and development tools", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Blang source file to WebAssembly
    #[command(alias = "c")]
    Compile {
        /// Input Blang source file
        input: PathBuf,

        /// Output WASM file
        #[arg(short, long)]
        output: PathBuf,

        /// Emit IR for debugging
        #[arg(long)]
        emit_ir: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,
    },

    /// Bundle Blang code with runtime for deployment
    #[command(alias = "b")]
    Bundle {
        /// Input Blang source file
        input: PathBuf,

        /// Output directory for bundle
        #[arg(long, default_value = "dist")]
        out_dir: PathBuf,

        /// Generate production optimized bundle
        #[arg(long)]
        release: bool,
    },

    /// Start development server with hot reload
    #[command(alias = "d")]
    Dev {
        /// Project directory to serve
        #[arg(default_value = ".")]
        dir: PathBuf,

        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.verbose {
        "blang_cli=debug,info"
    } else {
        "blang_cli=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Execute command
    match cli.command {
        Commands::Compile {
            input,
            output,
            emit_ir,
            opt_level,
        } => compile::run(input, output, emit_ir, opt_level),

        Commands::Bundle {
            input,
            out_dir,
            release,
        } => bundle::run(input, out_dir, release),

        Commands::Dev { dir, port, open } => dev::run(dir, port, open),
    }
}
