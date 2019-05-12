mod api;
mod config;
mod db;
mod file;
mod log;

use clap::{App, Arg, ArgGroup};

const ARG_LISTFILES: &str = "listfiles";
const ARG_GAME: &str = "game";
const ARG_UNNAMED: &str = "nxm_url";
const ARG_QUERY: &str = "query";

fn main() {
    let ver: &str = clap::crate_version!();
    let mut mandatory_args: Vec<&str> = Vec::new();
    mandatory_args.push(ARG_LISTFILES);
    mandatory_args.push(ARG_QUERY);
    mandatory_args.push(ARG_UNNAMED);

    let matches = App::new("dmodman")
        .version(ver)
        .author("dandels")
        .about("A third-party command line frontend to the Nexusmods API.")
        .arg(Arg::with_name(ARG_UNNAMED)
                .value_name("NXM_URL")
                .help("A nxm:// url to download.")
        )
        .arg(Arg::with_name(ARG_GAME)
                .short("g")
                .long("game")
                .value_name("GAME")
                .help("The game to manage. Required if the default game is not configured."),
        )
        .arg(Arg::with_name(ARG_QUERY)
                .short("q")
                .long("query")
                .value_name("MOD_ID")
                .help("Fetch information about a mod."),
        )
        .arg(Arg::with_name(ARG_LISTFILES)
                .short("d")
                .long("download")
                .value_name("MOD_ID")
                .help("List and download files of a mod."),
        )
        .group(ArgGroup::with_name("mandatory")
               .args(&mandatory_args)
               .required(true))
        .get_matches();

    if matches.is_present(ARG_UNNAMED) {
        if matches.value_of(ARG_UNNAMED).as_ref().unwrap().starts_with("nxm://") {
            api::nxmhandler::handle_nxm_url(matches.value_of(ARG_UNNAMED).unwrap());
            return
        } else {
            println!("Please provide a nxm url or specify an operation. See -h or --help for details, or consult the readme.");
            return
        }
    }

    let game: String;
    if matches.is_present(ARG_GAME) {
        game = matches.value_of(ARG_GAME).unwrap().to_string();
    } else {
        let res = config::get_game();
        match res {
            Ok(v) => game = v,
            Err(_) => {
                println!("The game to manage was neither specified nor found in the configuration file.");
                return
            }
        }
    }
    if matches.is_present(ARG_QUERY) {
        let q = matches.value_of(ARG_QUERY);
        let mod_id: u32 = q.unwrap().to_string().parse().expect("Invalid query. The provided argument must be a valid integer.");
        let mi = api::request::get_mod_info(&game, &mod_id).expect("Unable to get mod info");
        // Do something with query result
        println!("{}", mi.name);
    } else if matches.is_present(ARG_LISTFILES) {
        let q = matches.value_of(ARG_LISTFILES);
        let mod_id: u32 = q.unwrap().to_string().parse().unwrap();
        let mut fi: api::FileList = api::request::get_file_list(&game, &mod_id).expect("Unable to get file info");
        // Do something with dl results
        fi.files.sort();
        for file in fi.files.iter().filter(|x| x.category_name.as_ref().unwrap_or(&"".to_string()) != "OLD_VERSION") {
            println!("{:?} FILES", file.category_name.as_ref().unwrap_or(&"UNCATEGORIZED".to_string()));
            println!("{}, {}", file.name, file.version.as_ref().unwrap_or(&"".to_string()));
        }
        println!("-----------------------");
    } else {
    }
}
