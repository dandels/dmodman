use clap::Parser;

/// TUI downloader and update checker for Nexus Mods
#[deny(clippy::missing_docs_in_private_items)]
#[derive(Debug, Parser)]
pub struct Cli {
    /// Download file from nxm url
    pub nxm_url: Option<String>,

    /// Run in background.
    #[clap(long, short)]
    pub daemonize: bool
}
