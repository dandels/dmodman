mod api;
mod cli;
mod config;
mod db;
mod events;
mod extract;
mod logger;
mod nxm_socket;
mod prelude;
mod ui;
mod util;

use api::{Client, Downloads, Query};
use clap::Parser;
use cli::Cli;
use config::{Config, ConfigBuilder};
use db::Db;
use events::Events;
use logger::Logger;
use std::error::Error;
use std::io::ErrorKind;
use std::sync::LazyLock;

pub static CLI_OPTS: LazyLock<Cli> = LazyLock::new(Cli::parse);
pub static EVENTS: LazyLock<Events> = LazyLock::new(Events::new);
pub static LOGGER: LazyLock<Logger> = LazyLock::new(Logger::new);

/* dmodman acts as an url handler for nxm:// links in order for the "download with mod manager" button to work on
 * NexusMods.
 * If the program is invoked without argument, it starts the TUI unless another instance is already running.
 * If an nxm:// link is passed as an argument, we try to queue it in an already running instance. If none exists, we
 * start the TUI normally and queue the download.
 */

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse args before anything else
    let _ = CLI_OPTS.daemonize;

    let mut config = ConfigBuilder::load().unwrap_or_default().build()?;
    // let config = config_builder.build()?;

    if config.apikey().is_none() {
        if let Some(apikey) = ui::sso::start_apikey_flow() {
            config::save_apikey(&apikey)?;
            config.set_apikey_or_crash(apikey);
        } else {
            LOGGER.log("No API key configured. API connections are disabled.");
        }
    }

    let db = Db::new(config.clone()).await?;
    let client = Client::new(&config);
    let query = Query::new(db.clone(), client.clone());
    let downloads = Downloads::new(db.clone(), client.clone(), config.clone(), query.clone());

    // Try bind to /run/user/$uid. If it already exists then send any nxm:// link through the socket and quit.
    let nxm_socket = match nxm_socket::try_bind().await {
        Ok(nxm_socket) => nxm_socket,
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            println!("Another instance of dmodman is already running.");
            if let Some(nxm_str) = &CLI_OPTS.nxm_url {
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

    if let Some(nxm_str) = &CLI_OPTS.nxm_url {
        downloads.try_queue(nxm_str).await;
    }

    /* Start UI only if running interactively.
     * Otherwise we block the main thread with the listen loop so the program doesn't exit. */
    if CLI_OPTS.daemonize {
        nxm_socket::listen_for_downloads(nxm_socket, downloads).await;
    } else {
        {
            let downloads = downloads.clone();
            tokio::task::spawn(async move {
                nxm_socket::listen_for_downloads(nxm_socket, downloads).await;
            });
        }

        let lib = Lib {
            db,
            client,
            config,
            downloads,
            query,
        };

        ui::MainUI::start(lib).await;
    }

    Ok(())
}

pub struct Lib {
    config: Config,
    db: Db,
    client: Client,
    downloads: Downloads,
    query: Query,
}
