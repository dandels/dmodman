#![allow(dead_code)]
extern crate reqwest;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), Box<std::error::Error>> {
    let site = "https://api.nexusmods.com/v1/games";
    let game = "/morrowind";
    let modendpoint = "/46599.json";
    let _updateendpoint = "/mods/updated.json?period=1w";
    let endpoint: &str = modendpoint;
    let address = String::from(site) + game + endpoint;
    let apikey: String = file_to_string("apikey");

    let headers: HeaderMap = construct_headers(&apikey);
    let client = reqwest::Client::new();
    let mut resp = client
        .get(&address)
        .headers(headers)
        .send()?;
    let json: serde_json::Value = resp.json()?;
    //println!("{:#?}", resp);
    //println!("{:#?}", json);

    Ok(())
}

fn file_to_string(name: &str) -> String {
    let mut f = File::open(name).expect("Unable to open file");
    let mut contents: String = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents.trim().to_owned()
}

fn construct_headers(apikey: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let apiheader: HeaderValue = HeaderValue::from_str(apikey).unwrap();
    headers.insert("apikey", apiheader);
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key("apikey"));
    headers
}
