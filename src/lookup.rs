/* API responses are cached in order to reduce the number of API requests. There's a limit to how
 * many requests a user can perform per hour or day.
 * TODO: There's currently no way to clear the cache, bypass the cache, or to detect stale data.
 */

use super::utils;
use super::api::*;
use super::api::error::*;
use std::path::PathBuf;
use std::path::Path;
use std::str::FromStr;
use url::Url;

pub async fn handle_nxm_url(url_str: &str) -> Result<PathBuf, DownloadError> {
    let nxm = NxmUrl::from_str(url_str)?;
    let dl = DownloadLink::request(vec![&nxm.domain_name, &nxm.mod_id.to_string(), &nxm.file_id.to_string(), &nxm.query]).await?;
    let url: Url = Url::parse(&dl.location.URI)?;
    let file = request::download_mod_file(&nxm, &url).await?;
    check_dl_integrity(&nxm, &file).await?;
    Ok(file)
}

/* There is an "md5" value in the nxm url, but it's definitely not a valid md5sum. Instead we
 * calculate the md5 of the downloaded file and do a lookup for that hash. If the API lookup
 * returns a file with the same id, the download was succesful. The API might still give a 404 for
 * a file that exists.
 * The virus scan urls contain the sha256sums of the files, and could maybe be utilized.
 */
async fn check_dl_integrity(nxm: &NxmUrl, file: &Path) -> Result<Md5Search, Md5SearchError> {
    let md5search = by_md5(&nxm.domain_name, file).await?;
    if nxm.file_id == md5search.results.file_details.file_id {
        Ok(md5search)
    } else {
        Err(Md5SearchError::HashMismatch)
    }
}

pub async fn file_list(game: &str, mod_id: &u32) -> Result<FileList, RequestError> {
    match FileList::try_from_cache(&game, &mod_id) {
        Ok(v) => Ok(v),
        Err(_) => {
            let fl = FileList::request(vec![&game, &mod_id.to_string()]).await?;
            fl.save_to_cache(&game, &mod_id)?;
            Ok(fl)
        }
    }
}

pub async fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, RequestError> {
    match ModInfo::try_from_cache(&game, &mod_id) {
        Ok(v) => Ok(v),
        Err(_) => {
            let mi = ModInfo::request(vec![&game, &mod_id.to_string()]).await?;
            mi.save_to_cache(&game, &mod_id)?;
            Ok(mi)
        }
    }
}

pub async fn by_md5(game: &str, path: &Path) -> Result<Md5Search, Md5SearchError> {

    let md5 = utils::md5sum(path)?;
    let search = Md5Search::request(vec![&game, &md5]).await?;
    search.save_to_cache(&game, &search.results.r#mod.mod_id)?;

    if search.results.r#mod.domain_name != game {
        Err(Md5SearchError::GameMismatch)
    } else {
        Ok(search)
    }
}
