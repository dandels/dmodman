#![allow(dead_code)]
extern crate reqwest;

//use serde::{Deserialize, Serialize};
//use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<std::error::Error>> {
    let site = "https://api.nexusmods.com/v1/games".to_owned();
    let game = "/morrowind";
    let modendpoint = "46599.json";
    let _updateendpoint = "/mods/updated.json?period=1w";
    let endpoint: &str = modendpoint;
    let address = site.clone() + game + endpoint;
    let apikey: String = file_to_string("apikey");

    let client = reqwest::Client::new();
    let mut resp = client.get(&address)
        .headers(construct_headers(apikey))
        .send()?;

    let json: serde_json::Value = resp.json()?;

    println!("{:#?}", json);

    Ok(())
}

fn file_to_string(name: &str) -> String {
    let mut f = File::open(name).expect("Unable to open file");
    let mut contents: String = String::new();
    f.read_to_string(&mut contents);
    contents
}

fn construct_headers(apikey: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_static(apikey));
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));

    assert!(headers.contains_key(USER_AGENT));
    assert!(headers.contains_key("apikey"));

    headers
}
