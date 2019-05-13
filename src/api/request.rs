use crate::api::{DownloadLocation, FileList, ModInfo};
use crate::config;
use crate::db;
use crate::file;
use crate::log;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};
use std::path::PathBuf;
use url::Url;

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
const URL_API: &str = "https://api.nexusmods.com/v1/";
const URL_SUFFIX: &str = ".json";

pub fn get_file_list(game: &str, mod_id: &u32) -> Result<FileList, reqwest::Error> {
    match db::read_file_list(&game, &mod_id) {
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
    match db::read_mod_info(&game, &mod_id) {
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

pub fn download_mod_file(
    game: &str,
    mod_id: &u32,
    file_id: &u64,
    query: &str,
) -> Result<DownloadLocation, reqwest::Error> {
    match db::read_dl_loc(game, mod_id, file_id) {
        Ok(v) => Ok(v),
        Err(_v) => request_mod_file(game, mod_id, file_id, query),
    }
}

/* TODO unwind this spaghetti.
 */
pub fn request_mod_file(
    game: &str,
    mod_id: &u32,
    file_id: &u64,
    query: &str,
) -> Result<DownloadLocation, Error> {
    // Get dl link from API
    let endpoint = format!(
        "games/{}/mods/{}/files/{}/download_link.json?{}",
        &game, &mod_id, &file_id, &query
    );

    let dl: DownloadLocation = send_req(construct_request(&endpoint))?
        .json()
        .expect("Unable to parse list of download locations");

    db::save_dl_loc(game, mod_id, file_id, &dl)
        .expect("Unable to write to download location cache.");

    let dl_link = dl
        .location
        .get("URI")
        .expect("Invalid download location in cache")
        .as_str()
        .unwrap();
    let url: Url = Url::parse(dl_link).expect("Download link is not a valid URL");

    /* Get all the same mods' files from the api, and use that to determine the file name.
     * Url::parse url encodes the file name, . We query the API for the correct file name,
     * since we're going to use the information to check for updates anyways.
     */

    let fl = request_file_list(&game, &mod_id)?;
    let file: &super::FileInfo = fl
        .files
        .iter()
        .find(|x| x.file_id == *file_id)
        .expect("Unable to get file name from API");

    let file_name = &file.file_name;
    let file_location = config::download_dir(&game);
    let mut path = PathBuf::from(file_location);
    path.push(mod_id.to_string());
    file::create_dir_if_not_exist(&path.clone());
    path.push(&file_name);

    //TODO: We should maybe check the md5sum in the download url query parameters
    download_buffered(url, path)?;
    println!("Succesfully downloaded file.");
    Ok(dl)
}

fn download_buffered(url: Url, path: PathBuf) -> Result<(), reqwest::Error> {
    let mut buffer = std::fs::File::create(path).expect("Unable to save download to disk");
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let version = String::from("dmodman ") + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    let mut data = client.get(url).headers(headers).send()?;
    let _f = data.copy_to(&mut buffer);
    Ok(())
}

fn request_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let endpoint = format!("games/{}/mods/{}{}", &game, &mod_id, URL_SUFFIX);
    let builder = construct_request(&endpoint);
    let mut resp = send_req(builder)?;
    let mi: ModInfo = resp.json().expect("Unable to read response as mod info");
    db::save_mod_info(&mi).expect("Unable to write to db dir.");
    Ok(mi)
}

fn request_file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let endpoint = format!("games/{}/mods/{}/files{}", &game, &mod_id, URL_SUFFIX);
    let builder = construct_request(&endpoint);
    let mut resp = send_req(builder)?;
    let fl: FileList = resp.json().expect("Unable to read response as file list");
    db::save_file_list(&game, &mod_id, &fl).expect("Unable to write to db dir.");
    Ok(fl)
}

fn send_req(builder: reqwest::RequestBuilder) -> Result<Response, reqwest::Error> {
    let resp = builder.send()?;
    let headers = &resp.headers();
    log::info("Response headers:");
    log::append(&format!("{:#?}\n", headers));
    if !resp.status().is_success() {
        log::info(
            &(String::from("API response not OK, was: ")
                + resp.status().as_str()
                + " "
                + resp.url().as_str()),
        );
    }
    Ok(resp)
}

fn construct_request(endpoint: &str) -> reqwest::RequestBuilder {
    let url: String = String::from(URL_API) + endpoint;
    println!("Sending request to: {}", &url);
    let headers: HeaderMap = construct_headers();
    let client = reqwest::Client::new();
    let builder = client.get(&url).headers(headers);
    builder
}

fn construct_headers() -> HeaderMap {
    let apikey_header_name = "apikey";
    let apikey = config::api_key();
    let mut headers = HeaderMap::new();
    let apiheader: HeaderValue =
        HeaderValue::from_str(apikey.expect("No API key found in config").trim()).unwrap();
    let version = String::from("dmodman ") + clap::crate_version!();
    headers.insert(apikey_header_name, apiheader);
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key(apikey_header_name));
    headers
}
