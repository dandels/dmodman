Contributors welcome!
I'm open to ideas, feedback, and kind words. If you find that something doesn't work or is bothering you, please file an issue. I also never did end up using my program for the reason that I created it, so I can easily miss if something doesn't work well.

## Technical
* [Nexus API reference](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
* dmodman uses [ratatui](https://github.com/tui-rs-revival/ratatui) for the TUI.
* API responses are stored in `$XDG_DATA_HOME/dmodman/` (default `~/.local/share/dmodman`).

## Big things to do
A long-term project goal has been to use filesystem trickery to launch a modded game so that the mod files appear to the game to exist in the game directory. In this way the installed mods don't need to overwrite game data or eachother's data, and uninstallation or reordering of the installation order can be done cleanly. This is something like what ModOrganizer2 (and I suppose Vortex) does. The way to do this on Linux appears to be done with OverlayFS, of which there is the kernel one (requires root or setuid and has security implications) or fuse-overlayfs (some amount of performance penalty from Fuse?). This could however also be a separate program from dmodman, since the tool would mainly just need a list of directories and where to mount them.

I'm not particularly happy about working with working with Ratatui, though it's also a skill issue. The original reason for using a TUI is that Rust's UI library situation was famously bad, but fitting tabular data in a neat TUI has proven to be a challenge. I'm open to considering [Cursive](https://github.com/gyscos/cursive) as an alternative, or even a GUI, as long as it's fast. At the same time I'd like for the backend and frontend to be more cleanly separated from each other, and ideally you can just somewhat easily create a new frontend for the backend. I'm also personally not very interested in UI work, and getting stuck with it is a big part of why development has stalled. New backend features also depend on having a usable UI to use them with. Possible solutions include separating the backend into its own library, and/or doing a proper client/server split. This would also support the daemonization use case.

### General
- [ ] Split the project into Cargo workspaces so that compile times improve?
- [ ] Overlayfs support.
    - [ ] Some method to mount/umount the overlayfs. This could be as simple as a hotkey, or could be handled by an external program based on `load_order.txt`.
    - [ ] Decide between regular and/or fuse overlayfs.
- [ ] Consider in-app mod search? We can display and download mods based on API data (non-premium users need to visit Nexus for the download link). Dmodman already has an unused implementation for using the search API, so this is mostly a question of doing UI work + design.
- [ ] Fomod installer support. This requires dealing with the somewhat complex mod installation logic, as well as being able to show images either with terminal trickery or external programs.
- [ ] Add watching for file changes (with inotify?) while dmodman is running.

### UI
- [ ] Designing a good UI is hard. Consider which columns need showing, and perhaps add a way to configure/adjust what is shown.
    - The widths of table columns are hard to get right, the values are based on what works on my laptop.
    - Maybe it's more logical to show the installed mods as the default view.
- [ ] Implement Home, End and Page Up/Down keys.
- [ ] Fix the word-wrapping of list items somehow, since Ratatui doesn't support this.
- [x] Remember order of installed mods (can currently only be sorted in a text editor). The same order would be used by the overlayfs
    - [x] Rudimentary manual sort exists with J/K.
        - [ ] Improved sorting of table items
- [ ] Mouse support.
- [ ] Context menus that can be opened with right click or a key. This would help with the hotkey panel being cluttered with entries.

### File updates
- [x] Important: use the API to query mods that have been updated in the past 1
    month, combined with a timestamp for last update check
- [ ] Freezes UI during updates (does it still?). Needs threading a bit earlier in the code.
- [x] Show details of currently selected file in the UI. The border of the UI block can be rendered on.
- [ ] Permanently ignore updates (trivial to implement with a new enum variant)

### Archives
- [ ] Confirmation dialog for...
    - [x] Overwriting when extracting
    - [ ] File deletion (shift + delete to bypass confirmation?).
- [x] Show which mod a file belongs to.
- [ ] Import archives to dmodman using the md5search API.
- [x] Create metadata files into extracted directories (high prio)
- [ ] Allow extracting directly into the game data dir. This is a destructive operation and not allowed by default.

###  Downloads
- [ ] Delete cached download links when no longer valid/needed
- [ ] Query download urls without visiting the Nexus (Premium users only).

### Cache
- [x] Compression (zstd?) for the cache, easy to implement
- [ ] Some unneeded data is still kept around needlessly
    - [x] Prune old data from update lists when downloading them
    - [x] Avoid serializing unnecessary fields in FileDetails
    - [ ] Delete metadata if file is no longer referenced.

### Log
- [ ] Needs line wrapping. Depends on a WIP issue in the TUI library, but it hasn't progressed for years so a workaround is needed.
- [ ] The logger just writes to a file without ever cycling it, and rather needlessly too. Not having an API key creates log noise.
- [ ] Timestamps for the TUI a

### Configuration
- [ ] Color configuration?
- [ ] Maybe some users would like to ignore the API key prompt at start, using the program offline-only.

### Command line interface
- [x] Basic support for running in the background as a downloader.
    - [ ] Can't run at the same time as the TUI, since they would listen to the same socket. Fixing this would
    probably require a client/server architecture
- [ ] Set settings/profile from the CLI (the config uses the builder pattern so this is easy to implement)
- [ ] Help text (generated by a CLI-library?)
