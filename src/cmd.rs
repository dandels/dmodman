use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};

pub const ARG_ARCHIVE: &str = "archive";
pub const ARG_LISTFILES: &str = "listfiles";
pub const ARG_GAME: &str = "game";
pub const ARG_INTERACTIVE: &str = "interactive";
pub const ARG_MOD: &str = "query";
pub const ARG_UNNAMED: &str = "nxm_url";
pub const ARG_UPDATE: &str = "update";
pub const ARG_VERBOSITY: &str = "verbosity";

pub const VAL_GAME: &str = "GAME";
pub const VAL_FILE: &str = "FILE";
pub const VAL_MOD_ID: &str = "MOD_ID";
pub const VAL_UPDATE_TARGET: &str = "TARGET";
pub const VAL_VERBOSITY: &str = "VERBOSITY";

pub fn args() -> ArgMatches<'static> {
    let exclusive_args: Vec<&str> =
        vec![ARG_ARCHIVE, ARG_LISTFILES, ARG_MOD, ARG_UNNAMED, ARG_UPDATE];

    // TODO clap has nicer ways to define args
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
            Arg::with_name(ARG_ARCHIVE)
                .short("A")
                .long(ARG_ARCHIVE)
                .value_name(VAL_FILE)
                .help("Look up information about a mod archive."),
        )
        .arg(
            Arg::with_name(ARG_LISTFILES)
                .short("L")
                .long(ARG_LISTFILES)
                .value_name(VAL_MOD_ID)
                .help("List files of a mod."),
        )
        .arg(
            Arg::with_name(ARG_MOD)
                .short("M")
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
                .short("U")
                .long("update")
                .value_name(VAL_UPDATE_TARGET)
                .help("Check \"mod_id\" or \"all\" mods for updates."),
        )
        .group(
            ArgGroup::with_name("exclusive")
                .args(&exclusive_args)
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_VERBOSITY)
                .short("v")
                .long("verbosity")
                .value_name(VAL_VERBOSITY)
                .help("Sets the verbosity (log level). Possible values: error, warn, info, debug, trace."),
        )
        .arg(
            Arg::with_name(ARG_INTERACTIVE)
                .short("i")
                .long(ARG_INTERACTIVE)
                .help("Run interactively in the terminal."),
        )
    .get_matches();
    matches
}
