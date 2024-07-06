mod api;
mod cache;
mod config;
mod extract;
mod logger;
mod nxm_socket;
mod ui;
mod util;

use api::{Client, Downloads, Query};
use cache::Cache;
use config::{Config, ConfigBuilder};
use logger::Logger;
use std::env::args;
use std::error::Error;
use std::io::ErrorKind;
use std::sync::Arc;

/* dmodman acts as an url handler for nxm:// links in order for the "download with mod manager" button to work on
 * NexusMods.
 * If the program is invoked without argument, it starts the TUI unless another instance is already running.
 * If an nxm:// link is passed as an argument, we try to queue it in an already running instance. If none exists, we
 * start the TUI normally and queue the download.
 */

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut nxm_str_opt: Option<&str> = None;
    let mut is_interactive = true;

    let args: Vec<String> = args().collect();
    if args.len() > 2 {
        println!("Too many arguments. Invoke dmodman without arguments or with an nxm:// URL.");
        return Ok(());
    }
    if let Some(first_arg) = args.get(1) {
        if first_arg.starts_with("nxm://") {
            nxm_str_opt = Some(first_arg);
        } else if first_arg == "-d" {
            is_interactive = false;
        } else {
            // TODO use clap, this isn't true
            println!("Arguments are expected only when acting as an nxm:// URL handler.");
            return Ok(());
        }
    }

    /* We can't println in the TUI. Instead we use Logger which can log to a file and show messages in the TUI.
     * It calls println!() instead when running as a daemon. */
    let logger = Logger::new(is_interactive);

    let mut config: Config = ConfigBuilder::load(logger.clone())?.build()?;
    if config.apikey.is_none() {
        if let Some(apikey) = ui::sso::start_apikey_flow().await {
            config.apikey = Some(apikey);
            config.save_apikey()?;
        } else {
            logger.log("No API key configured. API connections are disabled.");
        }
    }
    let config = Arc::new(config);

    let cache = Cache::new(config.clone(), logger.clone()).await?;
    let client = Client::new(&config).await;
    let query = Query::new(cache.clone(), client.clone(), config.clone(), logger.clone());
    let downloads = Downloads::new(cache.clone(), client.clone(), config.clone(), logger.clone(), query.clone()).await;

    // Try bind to /run/user/$uid. If it already exists then send any nxm:// link through the socket and quit.
    let nxm_socket = match nxm_socket::try_bind().await {
        Ok(nxm_socket) => nxm_socket,
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            println!("Another instance of dmodman is already running.");
            if let Some(nxm_str) = nxm_str_opt {
                println!("Sending download to already running instance.");
                nxm_socket::send_msg(nxm_str).await.unwrap();
            }
            return Err(e.into());
        }
        Err(e) => {
            println!("Unable to bind to socket: {}", e);
            return Err(e.into());
        }
    };

    downloads.resume_on_startup().await;

    if let Some(nxm_str) = nxm_str_opt {
        downloads.try_queue(nxm_str).await;
    }

    /* Start UI only if running interactively.
     * Otherwise we block the main thread with the listen loop so the program doesn't exit. */
    if is_interactive {
        {
            let downloads = downloads.clone();
            let msgs = logger.clone();
            tokio::task::spawn(async move {
                nxm_socket::listen_for_downloads(nxm_socket, downloads, msgs).await;
            });
        }

        ui::MainUI::new(cache, client, config, downloads, logger, query).await.run().await;
    } else {
        nxm_socket::listen_for_downloads(nxm_socket, downloads, logger).await;
    }

    Ok(())
}
