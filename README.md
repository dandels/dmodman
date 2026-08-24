# dmodman
dmodman is a Linux-native TUI mod manager for [Nexus Mods](https://www.nexusmods.com/), written in Rust.
It functions as a handler for the nxm:// protocol, removing the need for manual downloads.

![The TUI with a list of mod files, a completed download, some files that could be updated, and some
messages. On the top is a bar with keybindings.](/screenshot.png)

## Notable features
- Downloads, extracts, and checks updates for mods.
- Nexus Mods SSO-integration.
- Fully multithreaded.
- Download state is remembered across program restarts
    * Expired downloads can be resumed by re-initiating the download from Nexus.
- API request cache to reduce traffic and speed up the program.
- Hash verification of completed downloads.
- Basic hjkl-navigation.

## Usage
* The first time dmodman is launched, an API key is generated for the user through Nexus's single sign-on.
    * API keys are stored in `$XDG_CONFIG_HOME/dmodman/apikey` and can be viewed in your [Nexus profile](https://www.nexusmods.com/users/myaccount?tab=api).
* The config is checked for in `$XDG_CONFIG_HOME` (~/.config/dmodman/config.toml). See the example [config.toml](/config.toml).
* Only one instance of dmodman can run at the same time.
* Mods are always downloaded to the current profile in order to support games with different editions, such as Skyrim.
* It's recommended to change the profile when modding a different game.
* Using the update checker:
    * Outdated files are marked with "!".
    * If a mod has some other new file, files are marked "?". (Can also be an update with broken metadata).
    * Update status is reset when a new file from that mod is downloaded.
    * Updates can be ignored until the next time a file in the mod is updated.
    * Tries to use cached data before sending an API request.
    * Could use more tests/testing and a code review
* `dmodman nxm://...` sends the url to the currently running instance. Useful for testing.

## Dependencies
* `libarchive 3.2.0` or higher to extract archives
* `xdg-utils` to `xdg-open` mod pages in the browser.

## Building
* dmodman works with the latest stable Rust toolchain.
* `git clone https://github.com/dandels/dmodman/`
* `cd dmodman`
* `cargo build --release` or `cargo run --release`

## Installation
The following steps are required for nxm scheme handling to work.
1. Add dmodman to your PATH. If developing, you can symlink to the build directory.
2. There is not yet a standard way to launch terminal apps from desktop files. You need to either:
    * Copy the included dmodman.desktop file to `~/.local/share/applications/dmodman.desktop` (or /usr/share/applications/). This file defaults to alacritty, and you need to configure it for your own terminal.
    * Alternatively use dmodman-terminal.desktop, which does not assume a specific terminal emulator. This is able to send downloads to an already running instance, but might fail at opening a terminal.
3. Run `update-desktop-database ~/.local/share/applications/` so that your browser is aware of the new protocol. This depends on `desktop-file-utils`, which presumably everyone has installed regardless of distribution.

## TODO & Contributing
There is an incomplete and somewhat up to date list of things that need doing in [CONTRIBUTING.md](/CONTRIBUTING.md).

## AI usage & policy
All code is written by humans (a single person as of writing, which would be fun to change). I do not accept contributions authored by LLMs.

## Logo
NexusMods requires users to have application-specific API keys. Having a logo was a prerequisite for being featured on
the NexusMods API page & getting dmodman-specific API keys via SSO. Therefore, here is a screenshot of an xterm with
font size 96, using the openly licensed [Space Mono](https://fonts.google.com/specimen/Space+Mono/) font:
![The logo of the program, spelling "% dmodman" with white on black in a zsh shell prompt with huge font size. The
cursor is hovered over the command, inverting the colors of the letter 'd'.](/dmodman.png)
