use crate::api::response::*;
use crate::config;
use crate::log;
use crate::utils;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};
use std::path::PathBuf;
use url::Url;

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
pub const API_URL: &str = "https://api.nexusmods.com/v1/";

pub fn md5search(game: &str, md5: &str) -> Result<Md5SearchResults, Error> {
    let endpoint = format!("games/{}/mods/md5_search/{}.json", &game, &md5);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder)?;
    match resp.error_for_status() {
        Ok(mut r) => Ok(r.json().expect("Unable to parse md5 lookup response.")),
        Err(r) => Err(r),
    }
}

pub fn dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &nxm.domain_name, &nxm.mod_id, &nxm.file_id, &nxm.query
    );
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder)?;
    match resp.error_for_status() {
        Ok(mut v) => Ok(v
            .json()
            .expect("Unable to read download link from response.")),
        Err(v) => Err(v),
    }
}

pub fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let endpoint = format!("games/{}/mods/{}.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder)?;
    match resp.error_for_status() {
        Ok(mut v) => Ok(v.json().expect("Unable to read response as mod info")),
        Err(v) => Err(v),
    }
}

pub fn file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let endpoint = format!("games/{}/mods/{}/files.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder)?;
    match resp.error_for_status() {
        Ok(mut v) => Ok(v.json().expect("Unable to read response as file list")),
        Err(v) => Err(v),
    }
}

pub fn download_mod_file(nxm: &NxmUrl, url: Url) -> Result<PathBuf, Error> {
    let file_name = utils::file_name_from_url(&url);
    let mut path = config::download_location_for(&nxm.domain_name, &nxm.mod_id);
    utils::mkdir_recursive(&path.clone());
    path.push(&file_name.to_string());

    download_buffered(url, &path)?;
    Ok(path)
}

fn download_buffered(url: Url, path: &PathBuf) -> Result<(), reqwest::Error> {
    let mut buffer = std::fs::File::create(path).expect("Unable to save download to disk");
    let builder = construct_request(url);
    let mut data = builder.send()?;
    let _f = data.copy_to(&mut buffer);
    Ok(())
}

fn send_req(builder: reqwest::RequestBuilder) -> Result<Response, reqwest::Error> {
    let resp = builder.send()?;
    let headers = &resp.headers();
    log::info("Response headers:");
    log::append(&format!("{:#?}\n", headers));
    log::info(&format!(
        "Got response: {} {:?}",
        resp.status().as_str(),
        resp.status().canonical_reason()
    ));
    Ok(resp)
}

fn construct_api_request(endpoint: &str) -> reqwest::RequestBuilder {
    let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
    let apiheader =
        HeaderValue::from_str(&config::api_key().expect("No API key found in config")).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", apiheader);
    let builder = construct_request(url).headers(headers);
    builder
}

fn construct_request(url: Url) -> reqwest::RequestBuilder {
    log::info(&format!("Sending request to: {}", &url));
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));

    let client = reqwest::Client::new();
    let builder = client.get(url).headers(headers);
    builder
}
