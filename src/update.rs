use super::api::FileList;
use super::config;
use super::request;
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
                Ok(mod_id) => match check_mod_dir(&path, &game, &mod_id) {
                    Ok(v) => {
                        if v {
                            updatable_mods.push(mod_id);
                        }
                    }
                    Err(e) => {
                        println!(
                            "Encountered error when checking mod directory {:?}: {:?}",
                            path, e
                        );
                    }
                },
                Err(_e) => {
                    println!("Ignoring non-mod directory in {}/{}", game, name);
                }
            }
        }
    }
    return updatable_mods;
}

fn check_mod_dir<'a>(moddir: &PathBuf, game: &str, mod_id: &u32) -> Result<bool, std::io::Error> {
    // TODO: move cache functionality to request.rs
    let mut filelist =
        request::file_list(&game, &mod_id).expect("Unable to fetch file list from API.");

    filelist.file_updates.sort_by_key(|a| a.uploaded_timestamp);

    for entry in fs::read_dir(moddir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) != Some("json") {
            if file_has_update(&filelist, &path, moddir.clone()) {
                return Ok(true);
            } else {
                /* This tells us that a specific file doesn't have updates, but some
                 * other file from the mod could have them. Since we don't have a
                 * convenient way of knowing which files are versions of the same
                 * file, we go through some unnecessary loop iterations.
                 */
            }
        }
    }
    Ok(false)
}

fn file_has_update(filelist: &FileList, path: &PathBuf, moddir: PathBuf) -> bool {
    /* The files are named like:
     *     Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z",
     *     ^file name                      ^mod  ^ver ^timestamp ^extension
     * This let's me lazily assume that we can rely on the file name being unqiue, and ignore the
     * file id's in the API response.
     * This should actually be rewritten to not rely on that behavior.
     */

    let mut fname = path.file_name().unwrap().to_str().unwrap();
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
                fname = &v.new_file_name;
                has_update = true;
            }
            None => {
                let mut f = moddir;
                f.push(fname);
                return !Path::new(&f).exists() && has_update;
            }
        }
    }
}
