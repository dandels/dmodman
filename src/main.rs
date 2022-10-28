mod api;
mod cache;
mod cmd;
mod config;
mod messages;
mod nxm_listener;
mod ui;
mod util;

use api::Client;
use cache::Cache;
use config::Config;
use messages::Messages;
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
    let matches = cmd::args();

    let mut nxm_str_opt: Option<&str> = None;
    let mut nxm_game_opt: Option<String> = None;

    if let Some(unnamed_arg) = matches.value_of(cmd::ARG_UNNAMED) {
        if unnamed_arg.starts_with("nxm://") {
            let nxm = api::NxmUrl::from_str(unnamed_arg).expect("Unable to parse nxm url, aborting.");
            nxm_str_opt = Some(unnamed_arg);
            nxm_game_opt = Some(nxm.domain_name);
        } else {
            println!("Invalid unnamed argument. See --help for usage.");
            return Ok(());
        }
    }

    let config  = Config::new(matches.value_of(cmd::ARG_GAME), nxm_game_opt).unwrap();
    let msgs = Messages::default();

    /* Check if another instance for the same game is already running. If it is, optionally queue the download, then
     * exit early.
     */
    let nxm_rx;
    match nxm_listener::queue_download_else_bind_to_socket(nxm_str_opt).await? {
        Some(v) => nxm_rx = v,
        None => return Ok(())
    }

    let cache = Cache::new(&config).await.unwrap();
    let client = Client::new(&cache, &config, &msgs).unwrap();

    if let Some(nxm_str) = nxm_str_opt {
        client.queue_download(nxm_str.to_string()).await;
    }

    nxm_listener::listen_for_downloads(&client, &msgs, nxm_rx);

    return ui::UI::init(cache, client, config, msgs).await?.run().await;
}
