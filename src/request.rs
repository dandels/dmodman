use crate::api::*;
use crate::config;
use crate::utils;
use log::{debug, error, info, trace};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};
use std::io::Write;
use std::path::PathBuf;
use url::Url;

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
pub const API_URL: &str = "https://api.nexusmods.com/v1/";

// The functions here have a lot of repetition. Figure out how to fix that.
pub async fn find_by_md5(game: &str, md5: &str) -> Result<Md5Search, Error> {
    let endpoint = format!("games/{}/mods/md5_search/{}.json", &game, &md5);
    let resp = send_api_request(&endpoint).await?.error_for_status();
    match resp {
        Ok(v) => v.json().await,
        Err(e) => Err(e)
    }
}

pub async fn nxm_dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &nxm.domain_name, &nxm.mod_id, &nxm.file_id, &nxm.query
    );
    let resp = send_api_request(&endpoint).await?;
    match resp.error_for_status() {
        Ok(r) => Ok(r
            .json().await
            .expect("Unable to read download link from response.")),
        Err(e) => Err(e),
    }
}

pub async fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let endpoint = format!("games/{}/mods/{}.json", &game, &mod_id);
    let resp = send_api_request(&endpoint).await?;
    match resp.error_for_status() {
        Ok(r) => Ok(r.json().await.expect("Unable to read response as mod info")),
        Err(e) => Err(e),
    }
}

pub async fn file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let endpoint = format!("games/{}/mods/{}/files.json", &game, &mod_id);
    match send_api_request(&endpoint).await?.error_for_status() {
        Ok(r) => Ok(r.json().await.expect("Unable to read response as file list")),
        Err(e) => Err(e),
    }
}

pub async fn download_mod_file(nxm: &NxmUrl, url: Url) -> Result<PathBuf, Error> {
    let file_name = utils::file_name_from_url(&url);
    let mut path = config::download_location_for(&nxm.domain_name, &nxm.mod_id);
    utils::mkdir_recursive(&path.clone());
    path.push(&file_name.to_string());

    download_buffered(url, &path).await?;
    Ok(path)
}

async fn download_buffered(url: Url, path: &PathBuf) -> Result<(), reqwest::Error> {
    let mut buffer = std::fs::File::create(path).expect("Unable to save download to disk");
    let builder = construct_request(url);
    let resp: reqwest::Response = builder.send().await?;
    match buffer.write_all(&resp.bytes().await?) {
        Ok(v) => {
            debug!("Download of {:?} succesful", path.file_name());
            Ok(v)
        },
        Err(e) => {
            error!("Download of {:?} finished with errors:\n{}", path.file_name(), e);
            Ok(())
        }
    }
}

async fn send_api_request(endpoint: &str) -> Result<Response, reqwest::Error> {
    let builder = construct_api_request(&endpoint);
    let resp = builder.send().await?;
    debug!("Response headers: {:#?}\n", resp.headers());
    debug!(
        "Got response: {} {:?}",
        resp.status().as_str(),
        resp.status().canonical_reason()
    );
    Ok(resp)
}

fn construct_api_request(endpoint: &str) -> reqwest::RequestBuilder {
    let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
    // This can probably be made more concise
    let apiheader =
        HeaderValue::from_str(&config::api_key().expect("No API key found in config")).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", apiheader);
    let builder = construct_request(url).headers(headers);
    builder
}

fn construct_request(url: Url) -> reqwest::RequestBuilder {
    debug!("Building request to: {}", &url);
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));

    let client = reqwest::Client::new();
    let builder = client.get(url).headers(headers);
    builder
}
