use super::api::request;
use super::api::response::{md5search, FileList, FileUpdate, Md5Search, Md5SearchResults};
use super::cache;
use super::{config, lookup};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// Checks a game for updates
pub fn check_game(game: &str) -> Result<(), std::io::Error> {
    // TODO clarify this explanation
    /* In order for updating to work, we need to know the file id and file name of the old
     * file. Rather than figure these out when checking for updates, we perform an md5search after
     * downloading a file, since they have all the information we need. This serves the dual
     * purpose of validating file downloads, because we don't know the encoding of the md5sum in the
     * download link.
     */

    let mut dls = config::downloads();
    dls.push(&game);
    for direntry in fs::read_dir(dls)? {
        let path = direntry?.path();
        //downloads/game/mod_id
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let parsed: Result<u32, std::num::ParseIntError> = name.parse();
            match parsed {
                Ok(mod_id) => check_mod_dir(&path, &game, &mod_id)?,
                Err(_e) => {
                    println!("Ignoring non-mod directory in {}/{}", game, name);
                    continue;
                }
            }
        } else {
            // TODO: import these files?
        }
    }
    return Ok(());
}

fn check_mod_dir(moddir: &PathBuf, game: &str, mod_id: &u32) -> Result<(), std::io::Error> {
    let fl = request::file_list(&game, &mod_id).expect("");
    cache::save_file_list(&game, &mod_id, &fl).expect("Unable to write to cache dir.");
    let mut file_infos: Vec<Md5Search> = Vec::new();
    let entries = fs::read_dir(moddir)?;
    for entry in entries {
        let path = &entry?.path();
        if !path.is_file() {
            println!("Ignoring non-file in mod directory: {:?}", &path);
            continue;
        }
        if path.extension().and_then(OsStr::to_str) != Some("json") {
            let resultopt = get_md5_result(&game, &mod_id, &path);
            if resultopt.is_none() {
                continue;
            }
            let results = resultopt.unwrap();
            let search = md5search::parse_results(&results.results.clone());
            file_infos.push(search);
        } else {
            //Ignore json files
        }
    }
    let _upd = check_files(&fl, &mut file_infos);
    Ok(())
}

fn check_files(files: &FileList, searches: &mut Vec<Md5Search>) {
    let mut old_files: Vec<&Md5Search> = Vec::new();
    let mut ignore: Vec<u64> = Vec::new();

    for search in searches {
        let file_id = search.md5_file_details.file_id;
        let mut files = files.file_updates.clone();
        files.sort_by_key(|a| a.uploaded_timestamp);
        for (i, update) in files.iter().enumerate() {
            if file_id == update.old_file_id {
                // todo figure out the logic on this one
                continue;
            }
        }
    }
}

fn get_md5_result(game: &str, mod_id: &u32, path: &PathBuf) -> Option<Md5SearchResults> {
    let mut metadata_file = path.clone();
    metadata_file.set_extension("json");

    if metadata_file.exists() {
        let mut contents = String::new();
        File::open(metadata_file)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let r: Result<Md5SearchResults, serde_json::Error> = serde_json::from_str(&contents);
        match r {
            Ok(v) => Some(v),
            Err(_e) => {
                println!("Unexpected json file in data directory: {:?}.json", path);
                return None;
            }
        }
    } else {
        return lookup::md5search(&game, &path);
    }
}
