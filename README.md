# dmodman
dmodman is a Linux-native TUI download manager and update checker for [NexusMods.com](https://www.nexusmods.com/) mods,
written in Rust. It can be registered as a handler for the nxm:// protocol, allowing the "Mod manager download"-button
to work on Nexus Mods.

While the software should be usable, it's considered alpha. Still, it should not crash or behave unexpectedly during
normal use. While it's not meant for general use until approved by Nexus Mods, testing and feedback is appreciated.

![The TUI with a list of mod files, a completed download, some files that could be updated, and some
messages.](/screenshot.png)

## Features
Incomplete list of existing & planned features:
- [x] Multithreaded downloads.
- [x] Pausing/resuming downloads.
- [x] Restore download state on startup.
    * Unpaused downloads automatically continue if their download links are still valid.
    * Expired downloads can be resumed by re-downloading them from the Nexus.
- [x] Sophisticated and fast (complicated and buggy) update algorithm that shouldn't have false positives like the
venerable MO2.
    * Outdated files are marked with a "!" in the "Flags" column of the files view.
    * If a mod has some other new file, files are marked with a "?". (Can also be an update with broken metadata).
    * Needs more testing.
- [x] Ignoring updates until the next time a file is updated.
- [x] Cache all responses to reduce API requests and speed up the program significantly.
- [x] API request counter
- [x] hjkl-navigation for vi aficionados.
- [ ] API key generation through Nexus SSO-integration.
- [ ] Hash verification of completed downloads (partially implemented). This was put on hold because of a
[bug](https://github.com/Nexus-Mods/web-issues/issues/1312) on Nexus's end.
- [ ] Importing existing downloads to dmodman.
- [ ] Download speed display.
- [ ] Tabbed views
- [ ] Better UI scaling
- [ ] Line wrap in the error message display.
- [ ] Querying download urls without visiting the Nexus (Premium users only).
- [ ] Tomato sauce (WIP) to go with the spaghetti code.

## Usage
* Copy the provided [config.toml](/config.toml) to `$XDG_CONFIG_HOME` (~/.config/dmodman/config.toml).
* The configuration file is located in `$XDG_CONFIG_HOME/dmodman/config.toml`.
    * Insert your API key (until key generation is supported).
    * API keys can be generated in your [Nexusmods profile](https://www.nexusmods.com/users/myaccount?tab=api).
    * Configure the game as shown in the config example.
* Only one instance of dmodman can run at the same time.
* Invoking `dmodman nxm://...` queues the download in the currently running instance. This is done automatically when
dmodman acts as an nxm handler.
* Mods are downloaded to the currently configured game, even if the file is from another game. This is intentional,
since mod files can be compatible with multiple game editions at once.
* API responses are cached in `$XDG_DATA_HOME/dmodman/` (defaults to `~/.local/share/dmodman`).

## Installation
* dmodman should work with any recent version of the Rust toolchain.
* While the head of the main branch is not guaranteed to be in a working state (until the first release at least) you
can install the program as follows:
* `git clone https://github.com/dandels/dmodman/`
* `cd dmodman`
* `cargo build --release` or `cargo run --release`
* It is recommended to add "dmodman" to your PATH, either by placing the binary
there or by symlinking to `target/release/dmodman` (or `target/debug/dmodman` if doing a debug build).
* Nxm url handling requires putting the dmodman.desktop file in `~/.local/share/applications` and dmodman being found in
the PATH environment variable.

## Technical
* [Nexus API reference](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
* dmodman uses [ratatui](https://github.com/tui-rs-revival/ratatui) for the TUI.
* While the program is written with Linux in mind, OS support should only be limited by the
[termion](https://docs.rs/termion/latest/termion/) terminal backend.
    * MacOS probably works fine, but I'm unable to test it.
    * Supporting Windows is not a goal, as they already have multiple choices of mod managers.
    * Enabling Windows support by replacing termion with [crossterm](https://docs.rs/crossterm/latest/crossterm/)
    shouldn't be too hard.

## Logo
NexusMods requires users to have application-specific API keys. Having a logo is a prerequisite for being featured on
the NexusMods API page & getting dmodman-specific API keys via SSO. Therefore, here is a screenshot of an xterm with
font size 96, using the openly licensed [Space Mono](https://fonts.google.com/specimen/Space+Mono/) font:
![The logo of the program, spelling "% dmodman" with white on black in a zsh shell prompt with huge font size. The
cursor is hovered over the command, inverting the colors of the letter 'd'.](/dmodman.png)
