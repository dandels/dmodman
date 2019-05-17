use crate::api::{DownloadLink, FileList, ModInfo, NxmUrl};
use crate::cache;
use crate::config;
use crate::log;
use crate::utils;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};
use std::path::PathBuf;
use url::Url;

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
pub const URL_API: &str = "https://api.nexusmods.com/v1/";

pub fn handle_nxm_url(url: &str) -> Result<(), reqwest::Error> {
    let nxm = NxmUrl::parse(url).expect("Malformed nxm url");
    if nxm.is_expired() {
        panic!("This nxm link has expired.");
    }
    let cached = cache::read_dl_link(&nxm);
    let dl: DownloadLink = cached.unwrap_or_else(|_| request_dl_link(&nxm).unwrap());
    let url: Url = Url::parse(
        dl.location
            .get("URI")
            .expect("Unable to read download link from API response.")
            .as_str()
            .unwrap(),
    )
    .expect("Download link is not a valid URL");
    return download_mod_file(nxm, url);
}

pub fn get_file_list(game: &str, mod_id: &u32) -> Result<FileList, reqwest::Error> {
    match cache::read_file_list(&game, &mod_id) {
        Ok(v) => {
            println!("Found file list in cache");
            Ok(v)
        }
        Err(_v) => {
            println!("Requesting file list from the API");
            return request_file_list(game, mod_id);
        }
    }
}

pub fn get_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, reqwest::Error> {
    match cache::read_mod_info(&game, &mod_id) {
        Ok(v) => {
            println!("Found mod info in cache");
            Ok(v)
        }
        Err(_v) => {
            println!("Requesting mod info from api");
            return request_mod_info(game, mod_id);
        }
    }
}

pub fn request_dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &nxm.domain_name, &nxm.mod_id, &nxm.file_id, &nxm.query
    );
    let mut resp = send_req(construct_api_request(&endpoint))?;
    let dl: DownloadLink;
    let json = resp.json();
    match json {
        Ok(v) => dl = v,
        Err(v) => {
            log::err(&format!("Unexpected response from the API: {}", v));
            panic!("Unexpected response from the API. See the log for more details.");
        }
    }
    cache::save_dl_link(&nxm, &dl).expect("Unable to write to download link cache.");
    Ok(dl)
}

pub fn download_mod_file(nxm: NxmUrl, url: Url) -> Result<(), Error> {
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
    download_buffered(url, path)?;
    println!("Succesfully downloaded file.");
    Ok(())
}

fn download_buffered(url: Url, path: PathBuf) -> Result<(), reqwest::Error> {
    let mut buffer = std::fs::File::create(path).expect("Unable to save download to disk");
    let builder = construct_request(url);
    let mut data = builder.send()?;
    let _f = data.copy_to(&mut buffer);
    Ok(())
}

fn request_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let endpoint = format!("games/{}/mods/{}.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder);
    match resp {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => {
                let mi: ModInfo = v.json().expect("Unable to read response as mod info");
                cache::save_mod_info(&mi).expect("Unable to write to cache dir.");
                Ok(mi)
            }
            Err(v) => Err(v),
        },
        Err(v) => return Err(v),
    }
}

fn request_file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let endpoint = format!("games/{}/mods/{}/files.json", &game, &mod_id);
    let builder = construct_api_request(&endpoint);
    let resp = send_req(builder);
    match resp {
        Ok(r) => match r.error_for_status() {
            Ok(mut v) => {
                let fl: FileList = v.json().expect("Unable to read response as file list");
                cache::save_file_list(&game, &mod_id, &fl).expect("Unable to write to cache dir.");
                Ok(fl)
            }
            Err(v) => Err(v),
        },
        Err(v) => return Err(v),
    }
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
    let url: Url = Url::parse(&(String::from(URL_API) + endpoint)).unwrap();
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
