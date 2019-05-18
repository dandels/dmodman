use super::api::request;
use super::cache;
use crate::api::response::*;
use url::Url;

pub fn handle_nxm_url(url: &str) -> Result<DownloadLink, reqwest::Error> {
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
    request::download_mod_file(&nxm, url).unwrap();
    cache::save_dl_link(&nxm, &dl).expect("Unable to write to download link cache.");
    Ok(dl)
}

pub fn file_list(game: &str, mod_id: &u32) -> Result<FileList, reqwest::Error> {
    match cache::read_file_list(&game, &mod_id) {
        Ok(fl) => {
            println!("Found file list in cache");
            cache::save_file_list(&game, &mod_id, &fl).expect("Unable to write to cache dir.");
            Ok(fl)
        }
        Err(_e) => {
            println!("Requesting file list from the API");
            return request::file_list(game, mod_id);
        }
    }
}

pub fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, reqwest::Error> {
    match cache::read_mod_info(&game, &mod_id) {
        Ok(mi) => {
            println!("Found mod info in cache");
            cache::save_mod_info(&mi).expect("Unable to write to cache dir.");
            Ok(mi)
        }
        Err(_e) => {
            println!("Requesting mod info from api");
            return request::mod_info(game, mod_id);
        }
    }
}

pub fn md5(game: &str, md5: &str) -> Md5Search {
    let results;
    match cache::read_md5search(game, md5) {
        Ok(r) => {
            results = r;
        }
        Err(_e) => {
            results = request::md5search(game, md5).unwrap();
            cache::save_md5search(&game, md5, &results).expect("Unable to write to cache.");
        }
    }
    return parse_results(results);
}
