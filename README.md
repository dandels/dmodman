# dmodman
dmodman is a Linux-native TUI download manager and update checker for [Nexus Mods](https://www.nexusmods.com/),
written in Rust. It can be registered as a handler for the nxm:// protocol, allowing the "Mod manager download"-button
to work on Nexus Mods.

![The TUI with a list of mod files, a completed download, some files that could be updated, and some
messages. On the top is a bar with keybindings.](/screenshot.png)

## Features
Incomplete list of existing and planned features.
- [x] Multithreaded downloads.
- [x] Pausing/resuming downloads.
- [x] Restore download state on startup.
    * Unpaused downloads automatically continue if their download links are still valid.
    * Expired downloads can be resumed by re-downloading them from the Nexus.
- [x] Sophisticated and fast (complicated and buggy) update algorithm that shouldn't have false positives like the
venerable MO2.
    * Outdated files are marked with a "!" in the "Flags" column of the files view.
    * If a mod has some other new file, files are marked with a "?". (Can also be an update with broken metadata). This
    flag is reset when a new file from that mod is downloaded.
    * Needs more testing.
- [x] Ignoring updates until the next time a file is updated.
- [x] Cache responses to reduce API requests and speed up the program significantly.
- [x] API request counter.
- [x] hjkl-navigation for vi aficionados.
- [x] API key generation through Nexus SSO-integration.
- [x] Hash verification of completed downloads. This had a
[bug](https://github.com/Nexus-Mods/web-issues/issues/1312) on Nexus's end, and is hopefully fixed now.
- [x] Opening mod page in browser.
- [ ] The UI is the bare minimum needed, and could use a lot of improvements.
- [ ] Importing already downloaded files to dmodman.
- [ ] Download speed display.
- [ ] Line wrap in the error message display.
- [ ] Querying download urls without visiting the Nexus (Premium users only).
- [ ] Tomato sauce to go with the occasional spaghetti code (WIP).

## Known issues
* The UI can crash the program when resizing the terminal window.

## Installation
* It is recommended to add "dmodman" to your PATH, either by placing the binary
there, or by symlinking to a release/debug binary (`target/release/dmodman` and `target/debug/dmodman`, respectively).
* Nxm url handling requires putting the dmodman.desktop file in `~/.local/share/applications` and dmodman being found in
PATH.
* dmodman depends on xdg-utils, as `xdg-open` is used for opening URLs.

## Usage
* Only one instance of dmodman can run at the same time.
* The first time dmodman is launched, an API key is generated for the user through Nexus's single sign-on.
    * API keys are stored in `$XDG_CONFIG_HOME/dmodman/apikey` and can be viewed in your [Nexusmods profile](https://www.nexusmods.com/users/myaccount?tab=api).
* The config file is checked for in `$XDG_CONFIG_HOME` (~/.config/dmodman/config.toml). See the example [config.toml](/config.toml).
    * Currently only supports configuring download location.
* Mods are downloaded to the same directory regardless of which game they belong to. This is intentional, since mod
files can be compatible with multiple game editions at the same time.
    * It's recommended to change the download directory and/or profile whenever modding a different game. For example,
    set the profile to "morrowind" if modding Morrowind.

## Building
* dmodman works with the latest stable Rust toolchain.
* `git clone https://github.com/dandels/dmodman/`
* `cd dmodman`
* `cargo build --release` or `cargo run --release`

## Technical
* [Nexus API reference](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
* API responses are cached in `$XDG_DATA_HOME/dmodman/` (defaults to `~/.local/share/dmodman`).
    * There is currently no automatic cache deletion.
    * The responses in `$game/file_lists` are used to display data and shouldn't be deleted.
* Invoking `dmodman nxm://...` queues the download in the currently running instance. Other command line arguments are
not supported.
* dmodman uses [ratatui](https://github.com/tui-rs-revival/ratatui) for the TUI.
* While the program is written with Linux in mind, OS support should mainly be limited by the
[termion](https://docs.rs/termion/latest/termion/) terminal backend.
    * MacOS probably works with few modifications, but I'm unable to test it. If there is demand and contributors, it
    can be supported.
    * Supporting Windows is not a goal, as they already have multiple choices of mod managers. However, enabling Windows
    support by replacing termion with [crossterm](https://docs.rs/crossterm/latest/crossterm/) shouldn't be too hard.

## Logo
NexusMods requires users to have application-specific API keys. Having a logo was a prerequisite for being featured on
the NexusMods API page & getting dmodman-specific API keys via SSO. Therefore, here is a screenshot of an xterm with
font size 96, using the openly licensed [Space Mono](https://fonts.google.com/specimen/Space+Mono/) font:
![The logo of the program, spelling "% dmodman" with white on black in a zsh shell prompt with huge font size. The
cursor is hovered over the command, inverting the colors of the letter 'd'.](/dmodman.png)
