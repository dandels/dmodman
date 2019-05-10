//extern crate reqwest;

use super::config;
use super::file;
use super::mod_info::ModInfo;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Error, Response};

// API reference:
// https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/Mods/get_v1_games_game_domain_name_mods_id.json
static URL_MOD_PREFIX: &str = "https://api.nexusmods.com/v1/games/";
static URL_SUFFIX: &str = ".json";

pub fn get_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let o = file::read_mod_info(&mod_id);

    if o.is_some() {
        return o;
    } else {
        return download_mod_info(game, mod_id);
    }
}

fn download_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let endpoint = format!("/mods/{}", &mod_id);
    let url: String = String::from(URL_MOD_PREFIX) + game + &endpoint + URL_SUFFIX;
    let headers: HeaderMap = construct_headers();
    let client = reqwest::Client::new();
    println!("Sending request to: {}", url);
    let resp: Result<Response, Error> = client.get(&url).headers(headers).send();
    // This can probably be refactored somehow, but I'm still figuring out Options and Results
    let mut r: Response;
    match resp {
        Ok(v) => r = v,
        Err(_) => {
            println!("Network request to \"{}\" failed", url);
            return None
        }
    }
    // TODO: handle network problems by informing user in some way
    println!("Got response: {}", r.status());
    if r.status().is_success() {
        // It's probably reasonable to crash if we can't parse the json
        let mi: ModInfo = r.json().ok().unwrap();
        file::save_mod_info(&mi).expect("Unable to write to db dir.");
        return Some(mi)
    } else {
        return None
    }
}

fn construct_headers() -> HeaderMap {
    let apikey_header_name = "apikey";
    let apikey = config::get_api_key();
    let mut headers = HeaderMap::new();
    let apiheader: HeaderValue = HeaderValue::from_str(apikey.trim()).unwrap();
    headers.insert(apikey_header_name, apiheader);
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key(apikey_header_name));
    headers
}
