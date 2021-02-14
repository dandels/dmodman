mod api;
mod cache;
mod cmd;
mod config;
mod logger;
mod lookup;
mod request;
mod test;
mod ui;
mod update;
mod utils;

use log::{error, info, trace, LevelFilter};
use std::path::PathBuf;
use tokio::runtime::Runtime;

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
            let file: PathBuf = rt.block_on(lookup::handle_nxm_url(url)).unwrap();
            info!("Finished downloading {:?}", file.file_name().unwrap());
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
        rt.block_on(list_files(is_interactive, &game, &mod_id));
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
                let mod_ids = rt.block_on(update::check_game(&game));
                for id in mod_ids {
                    info!("Mod has updates: {:?}", id);
                }
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

async fn list_files(is_interactive: bool, game: &str, mod_id: &u32) {
    let mut fl = lookup::file_list(&game, &mod_id).await.expect(ERR_MOD);
    if is_interactive {
        // Do something with dl results
        fl.files.sort();
        let headers = vec![
            "Filename".to_owned(),
            "Version".to_owned(),
            "Category".to_owned(),
            "Size (MiB)".to_owned(),
        ];
        let mut rows: Vec<Vec<String>> = Vec::new();
        for file in fl
            .files
            .iter()
            // TODO this unwrap crashed when the file didn't exist
            .filter(|x| x.category_name.as_ref().unwrap_or(&"".to_string()) != "OLD_VERSION")
        {
            let filename = file.name.to_owned();
            let ver = file.version.to_owned().unwrap_or("".to_string());
            let category = file.category_name.to_owned().unwrap_or("".to_string());
            let size = (file.size_kb * 1000 / (1024 * 1024)).to_string();
            let data: Vec<String> = vec![filename, ver, category, size];
            rows.push(data);
        }
        // TODO move to the interactivity check
        ui::term::init(headers, rows).unwrap();
    } else {
        for file in fl.files.iter() {
            info!("{}", file.name);
        }
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
