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
    /// Start HTTP server to serve markdown files
    #[command(name = "serve")]
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Directory to serve markdown files from
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}
