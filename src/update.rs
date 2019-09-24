use super::api::request;
use super::api::response::FileList;
use super::cache;
use super::config;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn check_game(game: &str) -> Vec<u32> {
    let mut dls = config::downloads();
    dls.push(&game);
    let mut updatable_mods: Vec<u32> = Vec::new();
    for direntry in fs::read_dir(dls).unwrap() {
        let path = direntry.unwrap().path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let parsed: Result<u32, std::num::ParseIntError> = name.parse();
            match parsed {
                Ok(mod_id) => updatable_mods.append(&mut check_mod_dir(&path, &game, &mod_id)),
                Err(_e) => {
                    println!("Ignoring non-mod directory in {}/{}", game, name);
                }
            }
        }
    }
    return updatable_mods
}

fn check_mod_dir<'a>(moddir: &PathBuf, game: &str, mod_id: &u32) -> Vec<u32> {
    // TODO: move cache functionality to request.rs
    let mut filelist =
        request::file_list(&game, &mod_id).expect("Unable to fetch file list from API.");
    cache::save_file_list(&game, &mod_id, &filelist).expect("Unable to write to cache dir.");

    filelist.file_updates.sort_by_key(|a| a.uploaded_timestamp);

    let mut ret: Vec<(u32)> = Vec::new();

    // this is pretty deeply nested and should be untangled
    match fs::read_dir(moddir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(file) => {
                        let path = file.path();
                        if path.is_file()
                            && path.extension().and_then(OsStr::to_str) != Some("json")
                        {
                            if file_has_update(&filelist, &path) {
                                ret.push(*mod_id);
                            }
                        }
                    }
                    Err(_e) => continue
                }
            }
        }
        Err(_e) => return ret
    }
    ret
}

fn file_has_update(filelist: &FileList, path: &PathBuf) -> bool {
    /* The files are named like:
     *     Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z",
     *     ^file name                      ^mod  ^ver ^timestamp ^extension
     * This let's me lazily assume that we can rely on the file name being unqiue, and ignore the
     * file id's in the API response.
     */

    let mut fname = path.to_str().unwrap();
    let mut has_update = false;
    loop {
        match filelist
            .file_updates
            .iter()
            .find(|x| x.old_file_name == fname)
        {
            Some(v) => {
                /* If new_file_name matches a file on disk, then there are multiple downloads of
                 * the same mod, and we're currently looking at the old version
                 */
                if Path::new(&v.new_file_name).exists() {
                    return false;
                }

                fname = &v.new_file_name;
                has_update = true;
            }
            None => return has_update,
        }
    }
}
