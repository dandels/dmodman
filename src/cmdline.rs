use std::env;
use super::config;

pub fn print_usage() {
    println!("dmodman [game] mod_id");
    println!("The game can also be specified in the configuration file.");
    println!("-h --help     Print this text");
}

pub fn check_args() -> Option<(String, u32)> {
    let args: Vec<_> = env::args().collect();
    let game: String;
    let mod_id: u32;
    let len = args.len();
    if len == 1 {
        print_usage();
        return None;
    } else if len == 2 {
        let arg: &str = &args[1];
        match arg {
            "-h" | "--help" => {
                print_usage();
                return None;
            }
            &_ => {
                game = config::get_game();
                mod_id = arg.parse().expect("mod_id is not a valid number");
                return Some((game, mod_id));
            }
        }
    } else if len == 3 {
        game = args[1].to_string();
        mod_id = args[2].parse().expect("mod_id is not a valid number");
        return Some((game, mod_id));
    }
    return None
}
