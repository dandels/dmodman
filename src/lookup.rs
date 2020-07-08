use super::{cache, request, utils};
use crate::api::*;
use log::{info, warn, error, trace};
use std::path::PathBuf;
use url::Url;

pub async fn handle_nxm_url(url_str: &str) -> Result<PathBuf, reqwest::Error> {
    let nxm = NxmUrl::parse(url_str).expect("Malformed nxm url");
    if nxm.is_expired() {
        panic!("This nxm link has expired.");
    }
    let dl: DownloadLink;
    match cache::read_dl_link(&nxm) {
        Ok(v) => dl = v,
        Err(_) => {
            dl = request::nxm_dl_link(&nxm).await?;
            cache::save_dl_link(&nxm, &dl).unwrap();
        }
    }
    let url: Url = Url::parse(&dl.location.URI).expect("Download link is not a valid URL");
    let file = request::download_mod_file(&nxm, url).await?;

    check_dl_integrity(&nxm, &file).await?;
    Ok(file)
}

/* There is an "md5" value in the nxm url, but it's definitely not a valid md5sum. Instead we
 * calculate the md5 of the downloaded file and do a lookup for that hash. If the API lookup
 * returns a file with the same id, the download was succesful.
 */
async fn check_dl_integrity(nxm: &NxmUrl, file: &PathBuf) -> Result<Option<Md5Search>, reqwest::Error> {
    let md5search = by_md5(&nxm.domain_name, &file).await?;
    let file_name = &file.file_name().unwrap().to_str().unwrap();
    match md5search {
        Some(s) => {
            trace!("{} {}", nxm.file_id, s.results.file_details.file_id);
            if nxm.file_id == s.results.file_details.file_id {
                return Ok(Some(s));
            } else {
                error!("Download was succesful, but an API lookup for this md5 returned a different file.\n);
                                    Ours:   {} ({})\n\
                                    Theirs: {} ({}))",
                       &file_name, nxm.file_id,
                       s.results.file_details.file_name, s.results.file_details.file_id
                       );
                return Ok(Some(s));
            }
        }
        None => {
            warn!(
                "Downloading of {:?} was succesful, but the API does not recognize any file with this md5sum. \
                 If you downloaded an old file, this message is harmless. Otherwise, this could indicate that \
                 the file was corrupted during the download.",
                &file_name
            );
            return Ok(None);
        }
    }
}

pub async fn file_list(game: &str, mod_id: &u32) -> Result<FileList, reqwest::Error> {
    match cache::read_file_list(&game, &mod_id) {
        Ok(fl) => {
            info!("Found file list in cache");
            Ok(fl)
        }
        Err(_e) => {
            info!("Requesting file list from the API");
            match request::file_list(game, mod_id).await {
                Ok(fl) => {
                    cache::save_file_list(&game, &mod_id, &fl).expect("Unable to write to cache dir.");
                    return Ok(fl);
                }
                Err(e) => {
                    error!("Unable to fetch file listing for mod {} for {}", mod_id, game);
                    Err(e)
                }
            }
        }
    }
}

pub async fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, reqwest::Error> {
    match cache::read_mod_info(&game, &mod_id) {
        Ok(mi) => {
            info!("Found mod info in cache");
            Ok(mi)
        }
        Err(_e) => {
            info!("Requesting mod info from api");
            let mi = request::mod_info(game, mod_id).await.expect("Unable to fetch mod info.");
            cache::save_mod_info(&mi).expect("Unable to write to cache dir.");
            return Ok(mi);
        }
    }
}

pub async fn by_md5(game: &str, path: &PathBuf) -> Result<Option<Md5Search>, reqwest::Error> {
    match cache::read_md5_search(path.clone()) {
        // This lookup is already cached
        Ok(s) => {
            /* Finding a mod from a different game when performing an md5 lookup could maybe happen
             * due to something the user has done. It could theoretically also mean an md5
             * collision on Nexuxmods.
             * TODO handle this case gracefully.
             */
            if s.results.r#mod.domain_name != game {
                error!(
                    "Error: Found mod file from another game ({}): {:?}.",
                    s.results.r#mod.domain_name, path,
                );
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
        // Not found in cache, request from API
        Err(_e) => {
            let md5 = utils::md5sum(path).unwrap();
            match request::find_by_md5(&game, &md5).await {
                Ok(s) => {
                    info!("Succesfully looked up {:?} via API.", path);
                    cache::save_md5_search(&game, &s).expect("Unable to write to cache.");
                    return Ok(Some(s))
                }
                Err(e) => {
                    error!("{}", e);
                    return Ok(None)
                }
            }
        }
    }
}
