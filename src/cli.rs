use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mdr")]
#[command(about = "A modern markdown renderer with built-in pager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the markdown file to render
    #[arg()]
    pub file: Option<String>,

    /// Show line numbers
    #[arg(short, long)]
    pub line_numbers: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a sample .mdr/config.toml in the current directory
    #[command(name = "init")]
    Init {
        /// Project directory
        #[arg(default_value = ".")]
        dir: String,
    },

    /// Start HTTP server to serve markdown files
    #[command(name = "serve")]
    Serve {
        /// Port to listen on（默认 8080，.mdr/config.toml 可覆盖）
        #[arg(short, long)]
        port: Option<u16>,

        /// Directory to serve markdown files from
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}
