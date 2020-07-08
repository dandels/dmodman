# dmodman
dmodman is a work in progress command line frontend to the
[Nexusmods API](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
It supports registering as an nxm url handler, allowing the "download with
manager"-button to work on the Nexus site. dmodman can check for updates to files that you have
downloaded with it, and also supports some API queries that have little practical
applications as of yet.

## Installation
dmodman is written in Rust, and should work with any recent version of the Rust
toolchain.
* `git clone https://github.com/dandels/dmodman/`
* `cd dmodman`
* `cargo build --release` or `cargo run --release`
* It is recommended to add "dmodman" to your PATH, by creating a symlink to the
executable, which can be found in `target/release/dmodman` (or
`target/debug/dmodman` if doing a debug build).

## Configuration
* dmodman attempts to adhere to the XDG directory specification, and uses
[dirs](https://lib.rs/crates/dirs) to find the appropriate locations for files.
* In order to use the program, you need a valid Nexusmods API key, which you can
generate in your [Nexusmods
profile](https://www.nexusmods.com/users/myaccount?tab=api). The API key needs
to be in in `$XDG_CONFIG_HOME/dmodman/apikey`.
* The default game to manage can be specified in
`$XDG_CONFIG_HOME/dmodman/game`. See [Usage](#Usage) for details on how to
provide the game name.
* Nxm url handling requires putting the dmodman.desktop file in
"\~/.local/share/applications", and adding
`x-scheme-handler/nxm=dmodman.desktop` to "\~/.local/share/applications/mimeapps.list".
* Downloaded files and API responses are currently stored in "$XDG_DATA_HOME/dmodman/".

## Usage
* The game name is expected to be in the same format as they are in
Nexusmods urls, since the name is passed as is to the APi.
    - [https://www.nexusmods.com/skyrimspecialedition/mods/266](https://www.nexusmods.com/skyrimspecialedition/mods/266) would become
    "skyrimspecialedition", for example.
    - Likewise, the mod id is the number in the url after "/mods/". Some operations
    rely on knowing this number, which isn't very user friendly, but that's how it
    works for now.

## Architecture
* It's spaghetti all the way down. I'm also still learning the language.
* Since there is a soft limit on daily API requests (see [API documentation](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/)), and this is very much alpha software,
API responses are cached and generally fetched from the cache on subsequent
requests. This reduces API requests when developing the software, and causes a
significant increase in speed. There is currently no built-in way to prune the
cache, other than manual deletion.
* The JSON in the API responses is deserialized with
[serde_json](https://docs.serde.rs/serde_json/). Returning JSON from a function
automatically converts it to one of the structs in "src/api/" if the return type
of the function is a compatible struct. See "src/request.rs" for examples.
