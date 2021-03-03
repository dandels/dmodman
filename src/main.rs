mod api;
mod cmd;
mod config;
mod db;
mod lookup;
mod nxm_socket;
mod test;
mod ui;
mod utils;

use std::io::{ Error, ErrorKind };
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = cmd::args();

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
    let nxm_receiver;

    // Try bind to /run/user/$uid/dmodman.socket in order to queue downloads for nxm:// urls
    match nxm_socket::listen(&uid) {
        Ok(v) => {
            nxm_receiver = v;
        }
        /* If the address is in use, either another instance is using it or a previous instance was killed without
         * closing it.
         */
        Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
            match nxm_socket::test_connection(&uid).await {
                // Another running instance is listening to the socket
                Ok(stream) => {
                    // If there's an nxm:// argument, queue it and exit
                    if let Some(nxm_str) = nxm_str_opt {
                        nxm_socket::queue_nxm_download(stream, nxm_str).await?;
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
                    nxm_socket::remove_existing(&uid).unwrap();
                    nxm_receiver = nxm_socket::listen(&uid).unwrap();
                },
                Err(e) => {
                    //TODO can we hit this case?
                    panic!("{}", e.to_string());
                }
            }
        },
        Err(e) => return Err(e)
    }

    if matches.is_present(cmd::ARG_UNNAMED) {
        let unnamed_arg = matches.value_of(cmd::ARG_UNNAMED).unwrap();
        if unnamed_arg.starts_with("nxm://") {
            match lookup::handle_nxm_url(unnamed_arg).await {
                Ok(file) => {
                    println!("Finished downloading {:?}", file.file_name().unwrap());
                }
                Err(e) => match e {
                    #[allow(unused_variables)]
                    api::error::DownloadError::Md5SearchError { source } => {
                        println!(
                            "Download succesful but file validation failed. This sometimes \
                                means the download is corrupted, but is usually caused by the md5 \
                                API being wonky."
                        )
                    }
                    _ => panic!("Download failed, {}", e),
                },
            }
        } else {
            println!(
                "Please provide an nxm url or specify an operation. See -h or -)-help for
                     details, or consult the readme."
            );
        }
        return Ok(())
    }

    let game: String;
    if let Some(g) = matches.value_of(cmd::ARG_GAME) {
        game = g.to_string();
    } else {
        game = nxm_game_opt.unwrap_or(
            config::game().expect("The game to manage was neither specified nor found in the configuration file.")
            );
    }

    ui::init(&game, nxm_receiver).await.unwrap();
    Ok(())
}
