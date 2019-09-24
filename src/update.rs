use super::api::request;
use super::cache;
use super::config;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

pub fn check_game(game: &str) -> Result<(), std::io::Error> {
    let mut dls = config::downloads();
    dls.push(&game);
    for direntry in fs::read_dir(dls)? {
        let path = direntry?.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let parsed: Result<u32, std::num::ParseIntError> = name.parse();
            match parsed {
                Ok(mod_id) => check_mod_dir(&path, &game, &mod_id)?,
                Err(_e) => {
                    println!("Ignoring non-mod directory in {}/{}", game, name);
                }
            }
        }
    }
    return Ok(());
}

fn check_mod_dir(moddir: &PathBuf, game: &str, mod_id: &u32) -> Result<(), std::io::Error> {
    // TODO: move cache functionality to request.rs
    let mut filelist =
        request::file_list(&game, &mod_id).expect("Unable to fetch file list from API.");
    cache::save_file_list(&game, &mod_id, &filelist).expect("Unable to write to cache dir.");

    let entries = fs::read_dir(moddir)?;
    filelist.file_updates.sort_by_key(|a| a.uploaded_timestamp);

    for file in entries {
        let path = &file?.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) != Some("json") {
            let mut new_file_exists = true;
            let mut has_update = false;
            let mut fname = path.to_str().unwrap();
            while new_file_exists {
                match filelist
                    .file_updates
                    .iter()
                    .find(|x| x.old_file_name == fname)
                {
                    Some(v) => {
                        fname = &v.new_file_name;
                        // if new_file_name matches a file on disk, we don't check updates for this file.
                        // put this in to its own function and use return to bail out in that case
                    }
                    None => new_file_exists = false,
                }
            }
        }
    }
    Ok(())
}
