use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "reposcope", about = "Terminal dashboard for local git repos")]
pub struct Cli {
    /// Print the selected repository path on quit for shell wrapper integration
    #[arg(long)]
    pub print_cd_target: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a git repository to tracking
    Add { path: String },
    /// Remove a tracked repository by name or path
    Remove { name_or_path: String },
    /// List tracked repositories
    List,
    /// Debug-scan a single repository (hidden, for Phase 2 verification)
    #[command(hide = true)]
    DebugScan { path: String },
    /// Debug-scan all registered repositories (hidden, for Phase 3 verification)
    #[command(hide = true)]
    DebugScanAll,
}
