use super::api::FileList;
use super::config;
use crate::api::{Requestable, Cacheable};
use log::debug;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::Error;

pub async fn check_game(game: &str) -> Result<Vec<u32>, Error> {
    let dls = config::download_dir(&game);
    let mut updatable_mods: Vec<u32> = Vec::new();
    for direntry in fs::read_dir(dls)? {
        println!("checking {:?} for updates", direntry);
        let path = direntry?.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let parsed: Result<u32, std::num::ParseIntError> = name.parse();
            match parsed {
                Ok(mod_id) => {
                    let mut filelist: FileList = FileList::request(vec![&game, &mod_id.to_string()]).await
                        .expect("Unable to fetch file list from API.");
                    filelist.save_to_cache(&game, &mod_id)?;
                    filelist.file_updates.sort_by_key(|a| a.uploaded_timestamp);

                    if check_mod_dir(&path, &filelist)? {
                        updatable_mods.push(mod_id);
                    } else {
                        println!("{} is up to date", name);
                    }
                }
                Err(_e) => {
                    debug!("Ignoring non-mod directory in {}/{}", game, name);
                }
            }
        }
    }
    return Ok(updatable_mods);
}

fn check_mod_dir<'a>(moddir: &PathBuf, filelist: &FileList) -> Result<bool, std::io::Error> {
    for entry in fs::read_dir(moddir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) != Some("json") {

            /* This tells us that a specific file doesn't have updates, but some other file from
             * the mod could have them. Since we don't have a convenient way of knowing which files
             * are versions of the same file, we go through some unnecessary loop iterations.
             */

            if file_has_update(&filelist, &path, moddir.clone()) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn file_has_update(filelist: &FileList, path: &PathBuf, moddir: PathBuf) -> bool {
    /* The files are named like:
     *     Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z",
     *     ^file name                      ^mod  ^ver ^timestamp ^extension
     * This lets us lazily assume that we can rely on the file name being unique, and ignore the
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
