use super::config;
use std::env;

pub fn print_usage() {
    println!("Usage: dmodman [game] mod_id");
    println!("The game can also be specified in the file {}/game, eg. \"morrowind\" (without the quotes)", config::get_config_dir().to_str().unwrap());
    println!("In order to use this software, you need to have generated a Nexusmods API key at https://www.nexusmods.com/users/myaccount?tab=api, and saved it in {}/apikey", config::get_config_dir().to_str().unwrap());
    println!("-h --help     Print this text");
}

pub fn check_args() -> Option<(String, u32)> {
    let args: Vec<_> = env::args().collect();
    let game: String;
    let mod_id: u32;
    let len = args.len();
    if len == 1 {
        print_usage();
        return None
    } else if len == 2 {
        let arg: &str = &args[1];
        match arg {
            "-h" | "--help" => {
                print_usage();
                return None
            }
            &_ => {
                game = config::get_game();
                mod_id = arg.parse().expect("mod_id is not a valid number");
                return Some((game, mod_id))
            }
        }
    } else if len == 3 {
        game = args[1].to_string();
        mod_id = args[2].parse().expect("mod_id is not a valid number");
        return Some((game, mod_id));
    }
    return None
}

pub fn get_game() -> String {
    let args: Vec<_> = env::args().collect();
    let len = args.len();
    if len == 2 {
        return config::get_game()
    }
    if len == 3 {
        return args[2].to_string()
    } else {
        panic!("Command line parameters are in an invalid state");
    }
}
