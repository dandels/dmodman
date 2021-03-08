mod api;
mod cmd;
mod config;
mod error_list;
mod db;
mod nxm_listener;
mod test;
mod ui;
mod util;

use api::Client;
use db::Cache;
use std::io::{ Error, ErrorKind };
use std::str::FromStr;
pub use self::error_list::ErrorList;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = cmd::args();

    let errors = ErrorList::default();

    let mut nxm_str_opt: Option<&str> = None;
    let mut nxm_game_opt: Option<String> = None;

    if let Some(unnamed_arg) = matches.value_of(cmd::ARG_UNNAMED) {
        if unnamed_arg.starts_with("nxm://") {
            let nxm = api::NxmUrl::from_str(unnamed_arg).unwrap();
            nxm_str_opt = Some(unnamed_arg);
            nxm_game_opt = Some(nxm.domain_name);
        } else {
            println!("Bogus unnamed argument. Bailing out.");
            return Ok(())
        }
    }

    let uid = users::get_current_uid();
    let mut nxm_rx;

    // Try bind to /run/user/$uid/dmodman.socket in order to queue downloads for nxm:// urls
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
                        return Ok(())
                    // otherwise just exit.
                    } else {
                        println!("Another instance of dmodman is already running.");
                        return Ok(())
                    }
                },
                /* Socket probably hasn't been cleanly removed. Remove it and bind to it.
                 */
                Err(ref e) if e.kind() == ErrorKind::ConnectionRefused => {
                    nxm_listener::remove_existing(&uid).unwrap();
                    nxm_rx = nxm_listener::listen(&uid).unwrap();
                },
                Err(e) => {
                    //TODO can we hit this case?
                    panic!("{}", e.to_string());
                }
            }
        },
        Err(e) => return Err(e)
    }

    let game = determine_active_game(&matches, nxm_game_opt);
     /* TODO ideally we would ask for the username/password and not require the user to create an apikey
     */
    let cache = Cache::new(&game).await.unwrap();
    let client = Client::new(&cache, &errors).unwrap();

    if let Some(nxm_str) = nxm_str_opt {
        Client::queue_download(client.clone(), nxm_str.to_string()).await;
    }

    // listen for nxm downloads
    {
        let client = client.clone();
        let _handle = tokio::task::spawn(async move {
            while let Some(nxm_result) = nxm_rx.recv().await {
                match nxm_result {
                    Ok(nxm_str) => {
                        Client::queue_download(client.clone(), nxm_str.to_string()).await;
                    },
                    Err(e) => { println!("{}", e.to_string()); }
                }
            }
        });
    }


    ui::init(&cache, &client, &errors).await.unwrap();
    Ok(())
}

/* Downloading mods from another game is a valid use case for Skyrim / Skyrim Special Edition users.
 * Order of precedence:
 * 1) Command line option
 * 2) Configuration file
 * 3) The game in the nxm url
 * ... otherwise bail out.
 * TODO: ask for game at runtime?
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
