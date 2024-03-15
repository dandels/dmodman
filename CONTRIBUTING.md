Contributors welcome!
I'm open to ideas, feedback, and kind words.

## Technical
* [Nexus API reference](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/).
* dmodman uses [ratatui](https://github.com/tui-rs-revival/ratatui) for the TUI.
* API responses are stored in `$XDG_DATA_HOME/dmodman/` (default `~/.local/share/dmodman`).

## Things to do

### General
- [x] Contextual keybinds
- [ ] Automatically sort table items
- [ ] Manually sort table items
- [ ] Remember table order.
- [ ] Color configuration?
- [ ] Implement Home, End and Page Up/Down keys.

### File updates
- [x] Important: use the API to query mods that have been updated in the past 1 month, combined with a timestamp for last update check
    - Freezes UI, Needs threading a bit earlier in the code
- [x] The displayed columns are arbitrary and not so relevant
- [x] Show details of currently selected file in the UI. The border of the UI block can be rendered on.
- [ ] Permanently ignore updates (trivial to implement with a new enum variant)
- [ ] Show archive connected to mod and vice versa
- [ ] Delete metadata if file is no longer tracked
- [x] The update lists can be quite long (13k lines in one case) and shouldn't be loaded to memory all the time.
    - [x] Needs compression on disk
    - Fixed. Unnecessary data is no longer kept.

### Archives
- [x] Extraction using libarchive
- [x] Popup dialog that asks for directory name
- [x] Configurable extract location
- [ ] Confirmation dialog (for overwriting when extracting and file deletion).
- [ ] Show which mod a file belongs to
- [ ] Import archives to dmodman using md5search.
- [ ] Fomod installer support
- [ ] Integrate into the tracked mods/updating UI
- [ ] Create metadata files into extracted directories (high prio)

###  Downloads
- [ ] Daemon that (un)mounts the overlayfs automatically
- [ ] Delete cached download links when no longer valid/needed
- [ ] Query download urls without visiting the Nexus (Premium users only).

### Cache
- [x] Compression (zstd?) for the cache, easy to implement
- [ ] A lot of stuff is needlessly kept around
    - Prune old data from update lists when downloading them
    - [x] Only a fraction of the fields of FileDetails are ever read

### Log
- [ ] Needs line wrapping. Depends on a WIP issue in the TUI library.

### Command line interface
- [x] Basic support for running in the background as a downloader. Can't run at the same time as the TUI,
    since they would listen to the same socket.
- [ ] Set settings/profile from the CLI (the config uses the builder pattern so this is easy to implement)
- [ ] Help text (generated by clap?)

### Other things
- [ ] Overlayfs support? This is needed by complex mod setups where files can overwrite each other.
    - [ ] Requires a way to reorder UI items (or implement as a text file with rows for directory order).
    - [ ] Decide between regular and/or fuse overlayfs
- [ ] In-app mod search, display and downloading (downloads for premium users only)?