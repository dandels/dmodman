mod api;
mod cache;
mod config;
mod messages;
mod nxm_listener;
mod ui;
mod util;

use api::{Client, Downloads};
use cache::Cache;
use config::Config;
use config::ConfigBuilder;
use messages::Messages;
use std::env::args;
use std::error::Error;
use std::str::FromStr;

/* dmodman acts as an url handler for nxm:// links in order for the "download with mod manager" button to work on
 * NexusMods.
 * - If the program is invoked without argument, it starts the TUI unless another instance is already running.
 * - If an nxm:// link is passed as an argument, we try to queue it in an already running instance. If none exists, we
 * start the TUI normally and queue the download.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut nxm_str_opt: Option<&str> = None;
    let mut game_opt: Option<String> = None;

    let args: Vec<String> = args().collect();
    if args.len() > 2 {
        println!("Too many arguments.");
        return Ok(());
    }
    if let Some(first_arg) = args.get(1) {
        if first_arg.starts_with("nxm://") {
            let nxm = api::NxmUrl::from_str(first_arg).expect("Unable to parse nxm url, aborting.");
            nxm_str_opt = Some(first_arg);
            game_opt = Some(nxm.domain_name);
        } else {
            println!("Arguments are expected only when acting as an nxm:// URL handler.");
            return Ok(());
        }
    }

    /* Check if another instance for the same game is already running.
     * If it is, we queue the download if one exists, then exit early. */
    let nxm_rx = match nxm_listener::queue_download_else_bind_to_socket(nxm_str_opt).await? {
        Some(v) => v,
        None => return Ok(()),
    };

    let msgs = Messages::default();

    let initialconfig = match ConfigBuilder::load() {
        Ok(mut ic) => {
            if ic.apikey.is_none() {
                if let Some(apikey) = ui::sso::start_apikey_flow().await {
                    ic = ic.apikey(apikey);
                } else {
                    return Ok(());
                }
            }
            // TODO configuring doesn't seem to be necessary anymore unless cross-mod downloading is disabled
            if ic.game.is_none() {
                if let Some(game) = game_opt {
                    ic = ic.game(game);
                } else {
                    panic!("TODO ask game");
                }
            }
            ic
        }
        Err(_e) => {
            panic!("Setting generation is not implemented.");
            // get apikey through SSO
            // show dialog to configure game
            // set rest to default
        }
    };

    // TODO wrap Config in an Arc or something, we're currently cloning it when we shouldn't be
    let config = initialconfig.build()?;

    let cache = Cache::new(&config).await?;
    let client = Client::new(&config, &msgs).await;
    let downloads = Downloads::new(&cache, &client, &config, &msgs).await;

    if let Some(nxm_str) = nxm_str_opt {
        let _ = downloads.queue(nxm_str.to_string()).await;
    }

    nxm_listener::listen_for_downloads(&downloads, &msgs, nxm_rx).await;

    ui::MainUI::new(cache, client, config, downloads, msgs).run().await
}
