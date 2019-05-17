mod api;
mod cache;
mod config;
mod log;
mod utils;

use clap::{App, AppSettings, Arg, ArgGroup};

const ARG_ARCHIVE: &str = "archive";
const ARG_LISTFILES: &str = "listfiles";
const ARG_GAME: &str = "game";
const ARG_QUERY: &str = "query";
const ARG_UNNAMED: &str = "nxm_url";
const ARG_UPDATE: &str = "update";

const VAL_GAME: &str = "GAME";
const VAL_FILE: &str = "FILE";
const VAL_MOD_ID: &str = "MOD_ID";

const ERR_GAME: &str =
    "The game to manage was neither specified nor found in the configuration file.";
const ERR_MOD_ID: &str = "Invalid argument. The specified mod id must be a valid integer.";
const ERR_QUERY: &str = "Unable to query mod info from API.";

fn main() {
    let mut exclusive_args: Vec<&str> = Vec::new();
    exclusive_args.push(ARG_ARCHIVE);
    exclusive_args.push(ARG_LISTFILES);
    exclusive_args.push(ARG_QUERY);
    exclusive_args.push(ARG_UNNAMED);
    exclusive_args.push(ARG_UPDATE);

    let matches = App::new(clap::crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(clap::crate_version!())
        .about("A third-party command line frontend to the Nexusmods API.")
        .arg(
            Arg::with_name(ARG_GAME)
                .short("g")
                .long("game")
                .value_name(VAL_GAME)
                .help("The game to manage. Required if the default game is not configured."),
        )
        .arg(
            Arg::with_name(ARG_ARCHIVE)
                .short("a")
                .long("archive")
                .value_name(VAL_FILE)
                .help("Look up information about a mod archive."),
        )
        .arg(
            Arg::with_name(ARG_LISTFILES)
                .short("l")
                .long("list")
                .value_name(VAL_MOD_ID)
                .help("List files of a mod."),
        )
        .arg(
            Arg::with_name(ARG_QUERY)
                .short("m")
                .long("mod")
                .value_name(VAL_MOD_ID)
                .help("Fetch information about a mod."),
        )
        .arg(
            Arg::with_name(ARG_UNNAMED)
                .value_name("nxm_url")
                .help("A nxm:// url to download."),
        )
        .arg(
            Arg::with_name(ARG_UPDATE)
                .short("u")
                .long("update")
                .value_name("target")
                .help("Check \"mod_id\", \"installed\" or \"all\" mods for updates."),
        )
        .group(
            ArgGroup::with_name("exclusive")
                .args(&exclusive_args)
                .required(true),
        )
        .get_matches();

    if matches.is_present(ARG_UNNAMED) {
        let url = matches.value_of(ARG_UNNAMED).unwrap();
        if url.starts_with("nxm://") {
            handle_nxm_url(&url);
        } else {
            println!("Please provide a nxm url or specify an operation. See -h or --help for details, or consult the readme.");
        }
        return;
    }

    let game: String = matches
        .value_of(ARG_GAME)
        .unwrap_or(&config::game().expect(ERR_GAME))
        .to_string();

    if matches.is_present(ARG_ARCHIVE) {
        let file_name = matches.value_of(ARG_ARCHIVE).unwrap();
        lookup_file(&game, &file_name);
        return;
    }

    if matches.is_present(ARG_LISTFILES) {
        let mod_id: u32 = matches
            .value_of(ARG_LISTFILES)
            .unwrap()
            .to_string()
            .parse()
            .expect(ERR_MOD_ID);
        list_files(&game, &mod_id);
        return;
    }

    if matches.is_present(ARG_QUERY) {
        let mod_id: u32 = matches
            .value_of(ARG_QUERY)
            .unwrap()
            .to_string()
            .parse()
            .expect(ERR_MOD_ID);
        query_mod_info(&game, &mod_id);
        return;
    }

    if matches.is_present(ARG_UPDATE) {
        return;
    }

    panic!("Reached end of main function without returning. This code should be unreachable.");
}

fn handle_nxm_url(url: &str) {
    let _dl_loc = api::request::handle_nxm_url(url).expect("Download failed");
    // We could print something useful here.
}

fn list_files(game: &str, mod_id: &u32) {
    let mut fi: api::FileList = api::request::get_file_list(&game, &mod_id).expect(ERR_QUERY);
    // Do something with dl results
    fi.files.sort();
    for file in fi
        .files
        .iter()
        .filter(|x| x.category_name.as_ref().unwrap_or(&"".to_string()) != "OLD_VERSION")
    {
        println!(
            "{:?} FILES",
            file.category_name
                .as_ref()
                .unwrap_or(&"UNCATEGORIZED".to_string())
        );
        println!(
            "{}, {}",
            file.name,
            file.version.as_ref().unwrap_or(&"".to_string())
        );
    }
    println!("-----------------------");
}

fn lookup_file(game: &str, file_name: &str) {
    let mut path = std::env::current_dir().expect("Current directory doesn't exist.");
    path.push(file_name);
    let md5 = utils::md5sum(&path).unwrap();
    println!("{}", md5);
}

fn query_mod_info(game: &str, mod_id: &u32) {
    let mi = api::request::get_mod_info(&game, &mod_id).expect(ERR_QUERY);
    // Do something with query result
    println!("{}", mi.name);
}
