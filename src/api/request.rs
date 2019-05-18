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
    match send_req(builder) {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => Ok(v.json().expect("Unable to parse md5 lookup response.")),
            Err(v) => Err(v),
        },
        Err(v) => Err(v),
    }
}

pub fn dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &nxm.domain_name, &nxm.mod_id, &nxm.file_id, &nxm.query
    );
    let builder = construct_api_request(&endpoint);
    match send_req(builder) {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => Ok(v
                .json()
                .expect("Unable to read download link from response.")),
            Err(v) => Err(v),
        },
        Err(v) => Err(v),
    }
}

pub fn mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let endpoint = format!("games/{}/mods/{}.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    match send_req(builder) {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => Ok(v.json().expect("Unable to read response as mod info")),
            Err(v) => Err(v),
        },
        Err(v) => return Err(v),
    }
}

pub fn file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let endpoint = format!("games/{}/mods/{}/files.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder);
    match resp {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => Ok(v.json().expect("Unable to read response as file list")),
            Err(v) => Err(v),
        },
        Err(v) => return Err(v),
    }
}

pub fn download_mod_file(nxm: &NxmUrl, url: Url) -> Result<(), Error> {
    let file_name = utils::file_name_from_url(&url);
    let file_location = config::downloads(&nxm.domain_name);
    let mut path = PathBuf::from(file_location);
    path.push(nxm.mod_id.to_string());
    utils::mkdir_recursive(&path.clone());
    path.push(&file_name.to_string());

    /* The md5sum in the download link is not a valid md5sum. It might be using some weird
     * encoding. Once the encoding is figured out, we can check the hash of the downloaded file.
     * Otherwise, we could calculate the md5sum ourselves and perform an API request to check it,
     * since the API only accepts normal md5sums.
     */
    download_buffered(url, &path)?;
    println!("Succesfully downloaded file.");
    Ok(())
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
    println!(
        "Got response: {} {:?}",
        resp.status().as_str(),
        resp.status().canonical_reason()
    );
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
    println!("Sending request to: {}", &url);
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));

    let client = reqwest::Client::new();
    let builder = client.get(url).headers(headers);
    builder
}
