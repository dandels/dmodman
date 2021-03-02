mod api;
mod cmd;
mod config;
mod db;
mod lookup;
mod test;
mod ui;
mod utils;

#[tokio::main]
async fn main() {
    let matches = cmd::args();

    /* TODO bind to a socket and handle all downloads through one instance
     */
    if matches.is_present(cmd::ARG_UNNAMED) {
        let url = matches.value_of(cmd::ARG_UNNAMED).unwrap();
        if url.starts_with("nxm://") {
            match lookup::handle_nxm_url(url).await {
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
        return;
    }

    let game: String = matches
        .value_of(cmd::ARG_GAME)
        .unwrap_or(&config::game().expect(
            "The game to manage was neither specified nor found in the configuration file.",
        ))
        .to_string();

    ui::init(&game).unwrap();
}
