mod api;
mod config;
mod file;
mod log;
mod utils;

use clap::{App, Arg, ArgGroup};
use std::io;

fn main() {
    let ver: &str = &utils::get_version();
    let mut opts: Vec<&str> = Vec::new();
    let arg_download = "download";
    let arg_query = "query";
    opts.push(arg_download);
    opts.push(arg_query);
    let matches = App::new("dmodman")
        .version(ver)
        .author("dandels")
        .about("A third-party command line frontend to the Nexusmods API")
        .arg(Arg::with_name("game")
                .short("g")
                .long("game")
                .value_name("GAME")
                .help("The game to manage. Required if the default game is not configured."),
        )
        .arg(Arg::with_name(arg_query)
                .short("q")
                .long(arg_query)
                .value_name("MOD_ID")
                .help("Fetch information about a mod."),
        )
        .arg(Arg::with_name(arg_download)
                .short("d")
                .long(arg_download)
                .value_name("MOD_ID")
                .help("List and download files of a mod."),
        )
        .group(ArgGroup::with_name("operation")
               .args(&opts)
               .required(true))
        .get_matches();

    let game: String;
    if matches.is_present("game") {
        game = matches.value_of("game").unwrap().to_string();
    } else {
        let g = config::get_game();
        match g {
            Ok(v) => game = v,
            Err(_) => {
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).unwrap();
                match buffer.trim_end() {
                    "" => return,
                    input => game = input.to_owned(),
                }
                config::set_game(&game).expect("Unable to write game setting to file.")
            }
        }
        println!("Using game from config: {}", game);
    }
    if matches.is_present(arg_query) {
        let q = matches.value_of(arg_query);
        let mod_id: u32 = q.unwrap().to_string().parse().unwrap();
        let mi = api::request::get_mod_info(&game, &mod_id).expect("Unable to get mod info");
        println!("{}", mi.name);
    } else if matches.is_present(arg_download) {
        let q = matches.value_of(arg_download);
        let mod_id: u32 = q.unwrap().to_string().parse().unwrap();
        let fi: api::DownloadList = api::request::get_download_list(&game, &mod_id).expect("Unable to get file info");
        for file in fi.files.iter() {
            println!("{}", file.name);
        }
    }
}
