mod api;
mod cache;
mod config;
mod messages;
mod nxm_listener;
mod ui;
mod util;

use api::Client;
use cache::Cache;
use config::Config;
use config::ConfigBuilder;
use messages::Messages;
use std::env::args;
use std::error::Error;
use std::str::FromStr;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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

    /* Check if another instance for the same game is already running. If it is, optionally queue the download, then
     * exit early.
     */
    let nxm_rx;
    match nxm_listener::queue_download_else_bind_to_socket(nxm_str_opt).await? {
        Some(v) => nxm_rx = v,
        None => return Ok(()),
    }

    let msgs = Messages::default();

    let initialconfig = match ConfigBuilder::load() {
        Ok(mut ic) => {
            if let None = ic.apikey {
                ic.apikey = gen_apikey(&msgs);
            }
            if let None = ic.game {
                if game_opt.is_some() {
                    ic.game = game_opt;
                } else {
                    panic!("TODO ask game");
                }
            }
            ic
        }
        Err(_e) => {
            panic!("Setting generation is not implemented.");
            // ask apikey
            // ask game
            // show user dialog to configure game, set rest to default
        }
    };

    let config = initialconfig.build()?;

    let cache = Cache::new(&config).await?;
    let client = Client::new(&cache, &config, &msgs).await?;

    if let Some(nxm_str) = nxm_str_opt {
        client.queue_download(nxm_str.to_string()).await;
    }

    nxm_listener::listen_for_downloads(&client, &msgs, nxm_rx).await;

    Ok(ui::ui::run(cache, client, config, msgs).await?)
}

fn gen_apikey(_msgs: &Messages) -> Option<String> {
    let mut generate_apikey = false;
    println!("You have not configured an API key.");
    println!("Would you like to create one? (This opens your browser.)");
    println!("[y]es, [n]o");
    /* Read y/n without waiting for the user to press return.
     * Entering raw mode messes with stdout, so we can't println until this scope ends.
     * Stdout is restored when _stdout is dropped. */
    {
        let _stdout = std::io::stdout().into_raw_mode().unwrap();
        let mut stdin = termion::async_stdin().keys();
        loop {
            if let Some(Ok(key)) = stdin.next() {
                match key {
                    Key::Char('y') => {
                        generate_apikey = true;
                        break;
                    }
                    Key::Char('n') => break,
                    _ => continue,
                }
            }
        }
    }
    if generate_apikey {
        // TODO begin Single Sign-On flow
        // Some("".to_string())
        return None;
    } else {
        return None;
    }
}
