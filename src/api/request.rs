use crate::api::{DownloadList, ModInfo};
use crate::config;
use crate::file;
use crate::log;
use crate::utils;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
const URL_MOD_PREFIX: &str = "https://api.nexusmods.com/v1/games/";
const URL_SUFFIX: &str = ".json";

pub fn get_download_list(game: &str, mod_id: &u32) -> Option<DownloadList> {
    let o = file::read_download_list(&game, &mod_id);

    if o.is_some() {
        return o;
    } else {
        return download_file_list(game, mod_id);
    }
}

pub fn get_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let o = file::read_mod_info(&game, &mod_id);

    if o.is_some() {
        return o;
    } else {
        return download_mod_info(game, mod_id);
    }
}

//TODO refactor into methods to avoid repetition
fn download_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let endpoint = format!("/mods/{}", &mod_id);
    let url: String = String::from(URL_MOD_PREFIX) + game + &endpoint + URL_SUFFIX;
    let headers: HeaderMap = construct_headers();
    let client = reqwest::Client::new();
    println!("Sending request to: {}", url);
    let resp: Result<Response, Error> = client.get(&url).headers(headers).send();
    match resp {
        Ok(mut v) => {
            let headers = &v.headers();
            log::info("Response headers:");
            log::append(&format!("{:#?}\n", headers));
            println!("Got response: {}", v.status());
            if v.status().is_success() {
                // It's probably reasonable to crash if we can't parse the json
                let mi: ModInfo = v.json().ok().unwrap();
                file::save_mod_info(&mi).expect("Unable to write to db dir.");
                return Some(mi)
            } else {
                log::err(&(String::from("API request not OK, was: ") + v.status().as_str()));
                return None
            }
        }
        Err(_) => {
            println!("Network request to \"{}\" failed", url);
            return None;
        }
    }
}

fn download_file_list(game: &str, mod_id: &u32) -> Option<DownloadList> {
    let endpoint = format!("/mods/{}/files", &mod_id);
    let url: String = String::from(URL_MOD_PREFIX) + game + &endpoint + URL_SUFFIX;
    let headers: HeaderMap = construct_headers();
    let client = reqwest::Client::new();
    println!("Sending request to: {}", url);
    let resp: Result<Response, Error> = client.get(&url).headers(headers).send();
    match resp {
        Ok(mut v) => {
            let headers = &v.headers();
            log::info("Response headers:");
            log::append(&format!("{:#?}\n", headers));
            println!("Got response: {}", v.status());
            if v.status().is_success() {
                // It's probably reasonable to crash if we can't parse the json
                let dl: DownloadList = v.json().unwrap();
                file::save_download_list(&game, &mod_id, &dl).expect("Unable to write to db dir.");
                return Some(dl)
            } else {
                log::err(&(String::from("API request not OK, was: ") + v.status().as_str()));
                return None
            }
        }
        Err(_) => {
            println!("Network request to \"{}\" failed", url);
            return None;
        }
    }

}

fn construct_headers() -> HeaderMap {
    let apikey_header_name = "apikey";
    let apikey = config::get_api_key();
    let mut headers = HeaderMap::new();
    let apiheader: HeaderValue = HeaderValue::from_str(apikey.trim()).unwrap();
    let version = String::from("dmodman") + &utils::get_version();
    headers.insert(apikey_header_name, apiheader);
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key(apikey_header_name));
    headers
}
