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

pub fn get_file_list(game: &str, mod_id: &u32) -> Option<FileList> {
    let o = db::read_file_list(&game, &mod_id);

    if o.is_some() {
        return o;
    } else {
        return request_file_list(game, mod_id);
    }
}

pub fn get_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let o = db::read_mod_info(&game, &mod_id);

    if o.is_some() {
        return o;
    } else {
        return request_mod_info(game, mod_id);
    }
}

/* TODO unwind this spaghetti.
 * Maybe don't perform an extra API request per downloaded file
 * Especially, implement an alternative way to determine the file name from the download url.
 * Url::parse url encodes the file name, which isn't desirable, so we query the API for the correct
 * file name, since we're going to use the information to check for updates anyways.
 */
pub fn download_mod_file(game: &str, mod_id: &u32, file_id: &u64, query: &str) {
    let endpoint = format!("games/{}/mods/{}/files/{}/download_link.json?{}", &game, &mod_id, &file_id, &query);
    let builder = construct_request(&endpoint);
    let file_location = config::get_download_dir(&game);
    let resp = send_req(builder);
    match resp {
        Some(mut v) => {
            let dls: DownloadLocation = v.json().expect("Unable to parse list of download locations");
            let dl_link = dls.location.get("URI").unwrap().as_str().unwrap();
            let fl = request_file_list(&game, &mod_id).unwrap();
            let file: &super::FileInfo = fl.files.iter().find(|x| x.file_id == *file_id).unwrap();
            let url: Url = Url::parse(dl_link).expect("Download link is not a valid URL");
            let file_name = &file.file_name;
            //We should maybe check the md5sum in the download link
            let mut path = PathBuf::from(file_location);
            path.push(mod_id.to_string());
            file::create_dir_if_not_exist(&path.clone());
            path.push(&file_name);

            let mut buffer = std::fs::File::create(path).expect("Unable to save download to disk");

            let client = reqwest::Client::new();
            let mut headers = HeaderMap::new();
            let version = String::from("dmodman ") + clap::crate_version!();
            headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
            let builder = client.get(url).headers(headers);
            match builder.send() {
                Ok(mut v) => {
                    let _f = v.copy_to(&mut buffer);
                },
                Err(v) => {
                    panic!(v)
                }
            }
            println!("Succesfully downloaded file.");
        },
        None => {
            println!("Unable to get download link from API");
            return
        }
    }

}

fn request_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let endpoint = format!("games/{}/mods/{}{}", &game, &mod_id, URL_SUFFIX);
    let builder = construct_request(&endpoint);
    let json = send_req(builder);
    match json {
        Some(mut v) => {
            let mi: ModInfo = v.json().expect("Unable to read response as mod info");
            db::save_mod_info(&mi).expect("Unable to write to db dir.");
            Some(mi)
        }
        None => None
    }
}

fn request_file_list(game: &str, mod_id: &u32) -> Option<FileList> {
    let endpoint = format!("games/{}/mods/{}/files{}", &game, &mod_id, URL_SUFFIX);
    let builder = construct_request(&endpoint);
    let json = send_req(builder);
    match json {
        Some(mut v) => {
            let fl: FileList = v.json().expect("Unable to read response as file list");
            db::save_file_list(&game, &mod_id, &fl).expect("Unable to write to db dir.");
            Some(fl)
        }
        None => None
    }
}
fn send_req(builder: reqwest::RequestBuilder) -> Option<Response> {
    let resp: Result<Response, Error> = builder.send();
    match resp {
        Ok(v) => {
            let headers = &v.headers();
            log::info("Response headers:");
            log::append(&format!("{:#?}\n", headers));
            if v.status().is_success() {
                return Some(v)
            } else {
                println!("Canceling request, expected status OK, was: {}", v.status());
                log::err(&(String::from("API response not OK, was: ") + v.status().as_str() + " " + v.url().as_str()));
                return None
            }
        }
        Err(_) => {
            println!("Network request failed");
            return None
        }
    }
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
    let apikey = config::get_api_key();
    let mut headers = HeaderMap::new();
    let apiheader: HeaderValue = HeaderValue::from_str(apikey.trim()).unwrap();
    let version = String::from("dmodman ") + clap::crate_version!();
    headers.insert(apikey_header_name, apiheader);
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key(apikey_header_name));
    headers
}
