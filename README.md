# dmodman
dmodman is a WIP TUI program for Linux written in Rust.

It supports downloading and updating mods via the [Nexusmods
API](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
It can be registered as an nxm url handler, allowing the "download with
manager"-button to work on the Nexus site.

Other features are planned, but development (when it happens) is currently
focusing on polishing before the first release. While it shouldn't destroy your
files, it's not meant for general use yet.

![The TUI with a list of mod files, a completed download, some files that
could be updated, and some messages.](/screenshot.png)

## Usage
* Only one instance of dmodman can run at the same time.
* Invoking dmodman with a nxm:// URL queues it in the currently running
  instance. If none exists, the application launches and begins downloading.
* Mods are downloaded to the currently configured instance, even if the file is
  from another game. This is intentional, since mod files can be compatible
  with multiple game editions at once.

## Installation
* dmodman should work with any recent version of the Rust toolchain.
* While no attempt is currently made to keep the head of the git branch in a
working state, you can install the program as follows:

* `git clone https://github.com/dandels/dmodman/`
* `cd dmodman`
* `cargo build --release` or `cargo run --release`
* It is recommended to add "dmodman" to your PATH, by creating a symlink to the
  executable, which can be found in `target/release/dmodman` (or
  `target/debug/dmodman` if doing a debug build).

## Configuration
* The configuration file is located in `$XDG_CONFIG_HOME/dmodman/config.toml`.
  [Example config.](/config.toml)
* In order to use the program, you need a valid Nexusmods API key. You
  can generate one your [Nexusmods profile](https://www.nexusmods.com/users/myaccount?tab=api).
* The game to manage is  be specified in `$XDG_CONFIG_HOME/dmodman/game`. See
  [Usage](#Usage) for details on how to provide the game name.
* Nxm url handling requires putting the dmodman.desktop file in
  `~/.local/share/applications`, and adding
  `x-scheme-handler/nxm=dmodman.desktop` to
  `~/.local/share/applications/mimeapps.list`.
* The download location defaults to `$XDG_DOWNLOAD_HOME/dmodman/<game>`, or
  ~/Downloads if `$XDG_DOWNLOAD_HOME` is unset.
* API responses are cached in `$XDG_DATA_HOME/dmodman/`.

## Logo
NexusMods requires users to have application-specific API keys. Having a logo
is a prerequisite for being featured on the NexusMods API page & users getting
dmodman-specific API keys. Therefore, here is a screenshot of an xterm with
font size 96, using a font with an [open license](https://fonts.google.com/specimen/Space+Mono/):
![The logo of the program, spelling "% dmodman" with white on black in a zsh
shell prompt with huge font size. The cursor is hovered over the of the
command, inverting the colors of the letter 'd'.](/dmodman.png)
