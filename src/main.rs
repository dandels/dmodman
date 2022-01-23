mod api;
mod cache;
mod cmd;
mod config;
mod errors;
mod nxm_listener;
mod ui;
mod util;

pub use self::errors::Errors;
use api::Client;
use cache::Cache;
use std::error::Error;
use std::io::ErrorKind;
use std::str::FromStr;
use tokio::sync::mpsc::Receiver;

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

    let nxm_rx;
    match queue_download_else_bind_to_socket(nxm_str_opt).await? {
        Some(v) => nxm_rx = v,
        None => return Ok(()),
    }

    let game = determine_active_game(&matches, nxm_game_opt);
    let errors = Errors::default();
    let cache = Cache::new(&game).await.unwrap();
    let client = Client::new(&cache, &errors).unwrap();

    /* We don't want to initialize the Cache or Client until we know we aren't exiting early, so the download can't be
     * queued before now.
     */
    if let Some(nxm_str) = nxm_str_opt {
        client.queue_download(nxm_str.to_string()).await;
    }

    listen_for_downloads(&client, &errors, nxm_rx);

    ui::init(&cache.file_details, &client, &errors).await?;
    Ok(())
}

/* Try bind to /run/user/$uid/dmodman.socket in order to queue downloads for nxm:// urls.
 * If the socket is already in use and the program was invoked with an nxm url, queue that download in the already
 * running instance and exit early.
 * If another instance is already running, we exit early.
 *
 * Returns Ok(None) if we we want to exit early, otherwise returns the mpsc receiver for the socket we bind to.
 */
async fn queue_download_else_bind_to_socket(
    nxm_str_opt: Option<&str>,
) -> Result<Option<Receiver<Result<String, std::io::Error>>>, std::io::Error> {
    let uid = users::get_current_uid();
    let nxm_rx;

    match nxm_listener::listen(&uid) {
        Ok(v) => {
            nxm_rx = v;
        }
        /* If the address is in use, either another instance is using it or a previous instance was killed without
         * closing it.
         */
        Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
            match nxm_listener::test_connection(&uid).await {
                // Another running instance is listening to the socket
                Ok(stream) => {
                    // If there's an nxm:// argument, queue it and exit
                    if let Some(nxm_str) = nxm_str_opt {
                        nxm_listener::send_msg(&stream, &nxm_str.as_bytes()).await?;
                        println!("Added download to already running instance: {}", nxm_str);
                        return Ok(None);
                    // otherwise just exit to avoid duplicate instances.
                    } else {
                        println!("Another instance of dmodman is already running.");
                        return Ok(None);
                    }
                }
                // Socket probably hasn't been cleanly removed. Remove it and bind to it.
                Err(ref e) if e.kind() == ErrorKind::ConnectionRefused => {
                    nxm_listener::remove_existing(&uid)?;
                    nxm_rx = nxm_listener::listen(&uid)?
                }
                /* Catchall for unanticipated ways in which the socket can break. Hitting this case should be
                 * unlikely.
                 */
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        }
        Err(e) => return Err(e),
    }
    Ok(Some(nxm_rx))
}

// Listen to socket for nxm links to download
fn listen_for_downloads(client: &Client, errors: &Errors, mut nxm_rx: Receiver<Result<String, std::io::Error>>) {
    let client = client.clone();
    let errors = errors.clone();
    let _handle = tokio::task::spawn(async move {
        while let Some(nxm_result) = nxm_rx.recv().await {
            match nxm_result {
                Ok(msg) => match api::NxmUrl::from_str(&msg) {
                    Ok(_) => client.queue_download(msg).await,
                    Err(_e) => errors.push(format!("Unable to parse string as a valid nxm url: {msg}")),
                },
                Err(e) => {
                    println!("{}", e.to_string());
                }
            }
        }
    });
}

/* Downloading mods from another game is a valid use case for Skyrim / Skyrim Special Edition users.
 * The game name is the same format as the url on nexusmods, eg. https://www.nexusmods.com/skyrimspecialedition/
 *
 * Order of precedence in which the game to manage is determined:
 * 1) Command line option
 * 2) Configuration file
 * 3) The game in the nxm url
 *
 * If none of these are set, bail out.
 * TODO: ask for game at runtime, and/or provide a readme that explains how to set it.
 */
fn determine_active_game(matches: &clap::ArgMatches, nxm_game_opt: Option<String>) -> String {
    if let Some(g) = matches.value_of(cmd::ARG_GAME) {
        g.to_string()
    } else if let Ok(configured_game) = config::game() {
        configured_game
    } else if let Some(nxm_game) = nxm_game_opt {
        nxm_game
    } else {
        // TODO handle this gracefully
        panic!("The game to manage was neither specified nor found in the configuration file.");
    }
}
