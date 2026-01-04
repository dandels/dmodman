use std::env::args;
use std::process;

use libc::{EXIT_FAILURE, EXIT_SUCCESS};

// TODO proper CLI lib
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VER: &str = env!("CARGO_PKG_VERSION");

pub struct CliOpts {
    pub is_interactive: bool,
    pub nxm_str_opt: Option<String>,
}

impl CliOpts {
    pub fn new() -> Self {
        let mut nxm_str_opt: Option<String> = None;
        let mut is_interactive = true;

        let help_text: String = format!(
            "{PKG_NAME} {PKG_VER}
    Invoke without arguments to run TUI in foreground.
     -d --daemonize\t\tRun in background.\
     nxm://<nxm_url>\t\tand download file from nxm url."
        );

        let args: Vec<String> = args().collect();
        if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
            println!("{help_text}");
            process::exit(EXIT_SUCCESS);
        }
        if args.len() > 2 {
            println!("Too many arguments. Valid arguments are \"-d\" or an nxm:// URL.");
            process::exit(EXIT_FAILURE);
        }
        if let Some(first_arg) = args.get(1) {
            if first_arg.starts_with("nxm://") {
                nxm_str_opt = Some(first_arg.to_string());
            } else if first_arg == "-d" || first_arg == "--daemonize" {
                is_interactive = false;
            } else {
                // TODO use clap, this isn't true
                println!("Arguments are expected only when acting as an nxm:// URL handler.");
                process::exit(EXIT_FAILURE);
            }
        }
        Self {
            is_interactive,
            nxm_str_opt,
        }
    }
}
