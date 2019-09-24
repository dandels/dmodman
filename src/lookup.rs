use super::{cache, request, utils};
use crate::api::*;
use std::path::PathBuf;
use url::Url;

pub fn handle_nxm_url(url: &str) -> Option<Md5SearchResults> {
    let nxm = NxmUrl::parse(url).expect("Malformed nxm url");
    if nxm.is_expired() {
        panic!("This nxm link has expired.");
    }
    let cached = cache::read_dl_link(&nxm);
    let dl: DownloadLink = cached.unwrap_or_else(|_| request::dl_link(&nxm).unwrap());
    let url: Url = Url::parse(
        dl.location
            .get("URI")
            .expect("Unable to read download link from API response.")
            .as_str()
            .unwrap(),
    )
    .expect("Download link is not a valid URL");
    let path = request::download_mod_file(&nxm, url).unwrap();
    cache::save_dl_link(&nxm, &dl).expect("Unable to write to download link cache.");
    // Best effort integrity check of the downloaded file
    let md5result = md5search(&nxm.domain_name, &path);
    let file_name = &path.file_name().unwrap().to_str().unwrap();
    match md5result {
        Some(r) => {
            let search = parse_results(&r.results.clone());
            println!("{} {}", nxm.file_id, search.md5_file_details.file_id);
            if nxm.file_id == search.md5_file_details.file_id {
                return Some(r);
            } else {
                println!("Download was succesful, but an API lookup for this md5 returned a different file.\n\
                                    Ours:   {} ({})\n\
                                    Theirs: {} ({}))",
                       &file_name, nxm.file_id,
                       search.md5_file_details.file_name, search.md5_file_details.file_id
                       );
                return Some(r);
            }
        }
        None => {
            println!(
                "Downloading of {:?} was succesful, but the API does not recognize any file with this md5sum. \
                 If you downloaded an old file, this message is harmless. Otherwise, this could indicate that \
                 the file was corrupted during the download.",
                &file_name
            );
            return None;
        }
    }
}

pub fn file_list(game: &str, mod_id: &u32) -> Result<FileList, reqwest::Error> {
    match cache::read_file_list(&game, &mod_id) {
        Ok(fl) => {
            println!("Found file list in cache");
            Ok(fl)
        }
        Err(_e) => {
            println!("Requesting file list from the API");
            let fl = request::file_list(game, mod_id).expect("Unable to fetch file listing");
            cache::save_file_list(&game, &mod_id, &fl).expect("Unable to write to cache dir.");
            return Ok(fl);
        }
    }
}

pub fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, reqwest::Error> {
    match cache::read_mod_info(&game, &mod_id) {
        Ok(mi) => {
            println!("Found mod info in cache");
            Ok(mi)
        }
        Err(_e) => {
            println!("Requesting mod info from api");
            let mi = request::mod_info(game, mod_id).expect("Unable to fetch mod info.");
            cache::save_mod_info(&mi).expect("Unable to write to cache dir.");
            return Ok(mi);
        }
    }
}

pub fn md5search(game: &str, path: &PathBuf) -> Option<Md5SearchResults> {
    match cache::read_md5search(&path) {
        Ok(r) => {
            let search = md5search::parse_results(&r.results.clone());
            if search.mod_info.domain_name != game {
                println!(
                    "Error: Found mod file from another game ({}): {:?}.",
                    search.mod_info.domain_name, path,
                );
                return None;
            }
            return Some(r);
        }
        Err(_e) => {
            let md5 = utils::md5sum(path).unwrap();
            let results;
            match request::md5search(&game, &md5) {
                Ok(r) => {
                    println!("Succesfully looked up {:?} via API.", path);
                    results = r;
                }
                Err(e) => {
                    println!("{}", e);
                    return None;
                }
            }
            cache::save_md5search(&game, &results).expect("Unable to write to cache.");
            Some(results)
        }
    }
}
