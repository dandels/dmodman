mod api;
mod cmd;
mod config;
mod db;
mod logger;
mod lookup;
mod test;
mod ui;
mod utils;

use log::{error, info, trace, LevelFilter};
use tokio::runtime::Runtime;
use db::update::UpdateChecker;

const ERR_MOD_ID: &str = "Invalid argument. The specified mod id must be a valid integer.";
const ERR_MOD: &str = "Unable to query mod info from API.";

fn main() {
    let matches = cmd::args();

    let mut is_interactive = false;

    if matches.is_present(cmd::ARG_INTERACTIVE) {
        is_interactive = true;
    }

    if matches.is_present(cmd::ARG_VERBOSITY) {
        logger::init(get_loglevel(matches.value_of(cmd::ARG_VERBOSITY))).unwrap();
    } else {
        logger::init(get_loglevel(None)).unwrap();
    }

    let rt = Runtime::new().unwrap();

    if matches.is_present(cmd::ARG_UNNAMED) {
        let url = matches.value_of(cmd::ARG_UNNAMED).unwrap();
        if url.starts_with("nxm://") {
            match rt.block_on(lookup::handle_nxm_url(url)) {
                Ok(file) => { info!("Finished downloading {:?}", file.file_name().unwrap()); }
                Err(e) => {
                    match e {
                        #[allow(unused_variables)]
                        api::error::DownloadError::Md5SearchError { source } => {
                            println!("Download succesful but file validation failed. This sometimes \
                                means the download is corrupted, but is usually caused by the md5 \
                                API being wonky.") }
                        _ => panic!("Download failed, {}", e)
                    }
                }
            }
        } else {
            error!(
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

    if matches.is_present(cmd::ARG_INTERACTIVE) {
        ui::init();
        return;
    }

    if matches.is_present(cmd::ARG_ARCHIVE) {
        trace!("Looking up mod archive");
        let file_name = matches.value_of(cmd::ARG_ARCHIVE).unwrap();
        rt.block_on(handle_md5_search(&game, &file_name));
        return;
    }

    if matches.is_present(cmd::ARG_LISTFILES) {
        let mod_id: u32 = matches
            .value_of(cmd::ARG_LISTFILES)
            .unwrap()
            .to_string()
            .parse()
            .expect(ERR_MOD_ID);
        rt.block_on(list_files(&game, &mod_id));
        return;
    }

    if matches.is_present(cmd::ARG_MOD) {
        let mod_id: u32 = matches
            .value_of(cmd::ARG_MOD)
            .unwrap()
            .to_string()
            .parse()
            .expect(ERR_MOD_ID);
        rt.block_on(query_mod_info(&game, &mod_id));
        return;
    }

    if matches.is_present(cmd::ARG_UPDATE) {
        match matches.value_of(cmd::VAL_UPDATE_TARGET) {
            Some("all") | None => {
                // TODO use results
                let updater = UpdateChecker::new(&game);
                let mod_ids = rt.block_on(updater.check_all());
            }
            Some(&_) => error!("Not implemented"),
        }
        return;
    }

    panic!("Reached end of main function without returning. This code should be unreachable.");
}

fn get_loglevel(verbosity: Option<&str>) -> LevelFilter {
    let mut loglevel = LevelFilter::Info;
    if verbosity.is_none() {
        info!("Verbosity not set");
        return loglevel;
    }

    let v: &str = &verbosity.unwrap().to_ascii_uppercase();
    match v {
        "TRACE" => loglevel = LevelFilter::Trace,
        "DEBUG" => loglevel = LevelFilter::Debug,
        "INFO" => loglevel = LevelFilter::Info,
        "WARN" => loglevel = LevelFilter::Warn,
        "ERROR" => loglevel = LevelFilter::Error,
        "OFF" => loglevel = LevelFilter::Off,
        _ => panic!("Invalid argument {} for verbosity.", v),
    }
    loglevel
}

async fn list_files(game: &str, mod_id: &u32) {
    let fl = lookup::file_list(&game, &mod_id).await.expect(ERR_MOD);
    for file in fl.files.iter() {
        info!("{}", file.name);
    }
}

async fn handle_md5_search(game: &str, file_name: &str) {
    let mut path = std::env::current_dir().expect("Current directory doesn't exist.");
    path.push(file_name);
    // TODO handle different error cases in frontend code
    let search =lookup::by_md5(game, &path).await.unwrap();
    trace!(
        "Mod name: {} \nFile name: {}",
        &search.results.r#mod.name,
        &search.results.file_details.name
    );
}

async fn query_mod_info(game: &str, mod_id: &u32) {
    let mi = lookup::mod_info(&game, &mod_id).await.expect(ERR_MOD);
    // Do something with query result
    info!("{:?}", mi);
}
