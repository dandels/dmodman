use crate::api::*;
use super::api::error::*;
use crate::cache;
use crate::config;
use crate::utils;
use log::{debug};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Response;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
pub const API_URL: &str = "https://api.nexusmods.com/v1/";

// The functions here have a lot of repetition. Figure out how to fix that.
pub async fn find_by_md5(game: &str, md5: &str) -> Result<Md5Search, DownloadError> {
    let endpoint = format!("games/{}/mods/md5_search/{}.json", &game, &md5);
    let resp = send_api_request(&endpoint).await?.error_for_status()?;
    let search: Md5Search = resp.json().await?;
    cache::save_md5_search(&game, &search)?;
    Ok(search)
}

pub async fn nxm_dl_link(nxm: &NxmUrl) -> Result<DownloadLink, DownloadError> {
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &nxm.domain_name, &nxm.mod_id, &nxm.file_id, &nxm.query
    );
    let resp = send_api_request(&endpoint).await?.error_for_status()?;
    let dl: DownloadLink = resp.json().await?;
    cache::save_dl_link(&nxm, &dl).unwrap();
    Ok(dl)
}

pub async fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, DownloadError> {
    let endpoint = format!("games/{}/mods/{}.json", &game, &mod_id);
    let resp = send_api_request(&endpoint).await?.error_for_status()?;
    let mi: ModInfo = resp.json().await?;
    cache::save_mod_info(&mi)?;
    Ok(mi)
}

pub async fn file_list(game: &str, mod_id: &u32) -> Result<FileList, DownloadError> {
    let endpoint = format!("games/{}/mods/{}/files.json", &game, &mod_id);
    let resp = send_api_request(&endpoint).await?.error_for_status()?;
    let fl: FileList = resp.json().await?;
    cache::save_file_list(&game, &mod_id, &fl)?;
    Ok(fl)
}

pub async fn download_mod_file(nxm: &NxmUrl, url: &Url) -> Result<PathBuf, DownloadError> {
    let file_name = utils::file_name_from_url(&url);
    let mut path = config::download_location_for(&nxm.domain_name, &nxm.mod_id);
    std::fs::create_dir_all(path.clone().to_str().unwrap())?;
    path.push(&file_name.to_string());
    download_buffered(&url, &path).await?;
    Ok(path)
}

async fn download_buffered(url: &Url, path: &PathBuf) -> Result<(), DownloadError> {
    let mut buffer = std::fs::File::create(path)?;
    let builder = build_request(&url);
    let resp: reqwest::Response = builder.send().await?;
    Ok(buffer.write_all(&resp.bytes().await?)?)
}

async fn send_api_request(endpoint: &str) -> Result<Response, DownloadError> {
    let builder = build_api_request(&endpoint)?;
    let resp = builder.send().await?;
    debug!("Response headers: {:#?}\n", resp.headers());
    debug!(
        "Got response: {} {:?}",
        resp.status().as_str(),
        resp.status().canonical_reason()
    );
    Ok(resp)
}

fn build_api_request(endpoint: &str) -> Result<reqwest::RequestBuilder, DownloadError> {
    let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
    let apikey = config::api_key()?;
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());
    let builder = build_request(&url).headers(headers);
    Ok(builder)
}

fn build_request(url: &Url) -> reqwest::RequestBuilder {
    debug!("Building request to: {}", &url);
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    let client = reqwest::Client::new();
    let builder = client.get(url.clone()).headers(headers);
    builder
}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::request;
    use crate::test;

    #[test]
    fn no_apikey() -> Result<(), DownloadError> {
        test::setup();
        println!("{:?}", dirs::config_dir());
        let req = request::build_api_request("http://localhost");
        if let Err(DownloadError::ApiKeyMissing) = req {
            return Ok(())
        }
        panic!("Expected error due to missing API key")
    }
}
