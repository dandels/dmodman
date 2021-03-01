use clap::{App, AppSettings, Arg, ArgMatches};

pub const ARG_GAME: &str = "game";
pub const ARG_UNNAMED: &str = "nxm_url";
pub const VAL_GAME: &str = "GAME";

pub fn args() -> ArgMatches<'static> {
    let matches: ArgMatches = App::new(clap::crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(clap::crate_version!())
        .about("A third-party command line frontend to the Nexusmods API.")
        .arg(
            Arg::with_name(ARG_GAME)
                .short("g")
                .long(ARG_GAME)
                .value_name(VAL_GAME)
                .help("The game to manage. Required if the default game is not configured."),
        )
        .arg(
            Arg::with_name(ARG_UNNAMED)
                .value_name("nxm_url")
                .help("A nxm:// url to download."),
        )
        .get_matches();
    matches
}
