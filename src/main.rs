mod api;
mod cache;
mod config;
mod file;
mod log;
mod utils;

use clap::{App, AppSettings, Arg, ArgGroup};

const ARG_LISTFILES: &str = "listfiles";
const ARG_GAME: &str = "game";
const ARG_QUERY: &str = "query";
const ARG_UNNAMED: &str = "nxm_url";

const ERR_GAME: &str =
    "The game to manage was neither specified nor found in the configuration file.";
const ERR_MOD_ID: &str = "Invalid argument. The specified mod id must be a valid integer.";
const ERR_QUERY: &str = "Unable to query mod info from API.";

fn main() {
    let mut exclusive_args: Vec<&str> = Vec::new();
    exclusive_args.push(ARG_LISTFILES);
    exclusive_args.push(ARG_QUERY);
    exclusive_args.push(ARG_UNNAMED);

    let matches = App::new(clap::crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(clap::crate_version!())
        .author("dandels")
        .about("A third-party command line frontend to the Nexusmods API.")
        .arg(
            Arg::with_name(ARG_GAME)
                .short("g")
                .long("game")
                .value_name("GAME")
                .help("The game to manage. Required if the default game is not configured."),
        )
        .arg(
            Arg::with_name(ARG_UNNAMED)
                .value_name("NXM_URL")
                .help("A nxm:// url to download."),
        )
        .arg(
            Arg::with_name(ARG_LISTFILES)
                .short("l")
                .long("list")
                .value_name("MOD_ID")
                .help("List files of a mod."),
        )
        .arg(
            Arg::with_name(ARG_QUERY)
                .short("q")
                .long("query")
                .value_name("MOD_ID")
                .help("Fetch information about a mod."),
        )
        .group(
            ArgGroup::with_name("exclusive")
                .args(&exclusive_args)
                .required(true),
        )
        .get_matches();

    if matches.is_present(ARG_UNNAMED) {
        if matches
            .value_of(ARG_UNNAMED)
            .as_ref()
            .unwrap()
            .starts_with("nxm://")
        {
            let _dl_loc = api::request::handle_nxm_url(matches.value_of(ARG_UNNAMED).unwrap())
                .expect("Download failed");
            return;
        } else {
            println!("Please provide a nxm url or specify an operation. See -h or --help for details, or consult the readme.");
            return;
        }
    }

    let game: String = matches
        .value_of(ARG_GAME)
        .unwrap_or(&config::game().expect(ERR_GAME))
        .to_string();

    if matches.is_present(ARG_QUERY) {
        let q = matches.value_of(ARG_QUERY);
        let mod_id: u32 = q.unwrap().to_string().parse().expect(ERR_MOD_ID);
        let mi = api::request::get_mod_info(&game, &mod_id).expect(ERR_QUERY);
        // Do something with query result
        println!("{}", mi.name);
        return;
    }

    if matches.is_present(ARG_LISTFILES) {
        let q = matches.value_of(ARG_LISTFILES);
        let mod_id: u32 = q.unwrap().to_string().parse().unwrap();
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
    } else {
        panic!("This code should be unreachable");
    }
}
