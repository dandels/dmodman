mod api;
mod cache;
mod config;
mod log;
mod lookup;
mod request;
mod ui;
mod update;
mod utils;

use clap::{App, AppSettings, Arg, ArgGroup};

const ARG_ARCHIVE: &str = "archive";
const ARG_LISTFILES: &str = "listfiles";
const ARG_GAME: &str = "game";
const ARG_QUERY: &str = "query";
const ARG_UNNAMED: &str = "nxm_url";
const ARG_UPDATE: &str = "update";
const ARG_UPDATE_TARGET: &str = "target";

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
                .value_name(ARG_UPDATE_TARGET)
                .help("Check \"mod_id\" or \"all\" mods for updates."),
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
            match lookup::handle_nxm_url(url) {
                Some(_v) => println!("Download complete"),
                None => println!("Download finished with warnings."),
            }
        } else {
            println!(
                "Please provide a nxm url or specify an operation. See -h or --help for
                     details, or consult the readme."
            );
        }
        return;
    }

    let game: String = matches
        .value_of(ARG_GAME)
        .unwrap_or(&config::game().expect(ERR_GAME))
        .to_string();

    if matches.is_present(ARG_ARCHIVE) {
        let file_name = matches.value_of(ARG_ARCHIVE).unwrap();
        md5search(&game, &file_name);
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
        match matches.value_of(ARG_UPDATE_TARGET) {
            Some("all") | None => {
                let mod_ids = update::check_game(&game);
                for id in mod_ids {
                    println!("Mod has updates: {}", id);
                }
            }
            Some(&_) => println!("Not implemented"),
        }
        return;
    }

    panic!("Reached end of main function without returning. This code should be unreachable.");
}

fn list_files(game: &str, mod_id: &u32) {
    let mut fl = lookup::file_list(&game, &mod_id).expect(ERR_QUERY);
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
        .filter(|x| x.category_name.as_ref().unwrap_or(&"".to_string()) != "OLD_VERSION")
    {
        let filename = file.name.to_owned();
        let ver = file.version.to_owned().unwrap_or("".to_string());
        let category = file.category_name.to_owned().unwrap_or("".to_string());
        let size = (file.size_kb * 1000 / (1024 * 1024)).to_string();
        let data: Vec<String> = vec![filename, ver, category, size];
        rows.push(data);
    }
    ui::term::init(headers, rows).unwrap();
}

fn md5search(game: &str, file_name: &str) {
    let mut path = std::env::current_dir().expect("Current directory doesn't exist.");
    path.push(file_name);
    let search = api::md5search::parse_results(&lookup::md5search(game, &path).unwrap().results);
    println!(
        "Mod name: {} \nFile name: {}",
        &search.mod_info.name, &search.md5_file_details.name
    );
}

fn query_mod_info(game: &str, mod_id: &u32) {
    let mi = lookup::mod_info(&game, &mod_id).expect(ERR_QUERY);
    // Do something with query result
    println!("{}", mi.name);
}
