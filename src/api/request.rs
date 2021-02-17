use crate::api::NxmUrl;
use crate::api::search::*;
use crate::api::error::RequestError;
use crate::{config, utils};
use log::{debug};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Response;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

/* API reference:
 * https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0
 */
pub const API_URL: &str = "https://api.nexusmods.com/v1/";
pub const SEARCH_URL: &str = "https://search.nexusmods.com/mods";

pub async fn mod_search(query: String) -> Result<Search, RequestError> {
    let base: Url = Url::parse(SEARCH_URL).unwrap();
    let url = base.join(&query).unwrap();
    let builder = build_request(&url);
    let resp: reqwest::Response = builder.send().await?;
    let ret = resp.json().await?;
    Ok(ret)
}

pub async fn download_mod_file(nxm: &NxmUrl, url: &Url) -> Result<PathBuf, RequestError> {
    let file_name = utils::file_name_from_url(&url);
    let mut path = config::download_dir(&nxm.domain_name);
    std::fs::create_dir_all(path.clone().to_str().unwrap())?;
    path.push(&file_name.to_string());
    download_buffered(&url, &path).await?;
    Ok(path)
}

async fn download_buffered(url: &Url, path: &PathBuf) -> Result<(), RequestError> {
    let mut buffer = std::fs::File::create(path)?;
    println!("downloading to... {:?}", path.as_os_str());
    let builder = build_request(&url);
    let resp: reqwest::Response = builder.send().await?;
    Ok(buffer.write_all(&resp.bytes().await?)?)
}

pub async fn send_api_request(endpoint: &str) -> Result<Response, RequestError> {
    let builder = build_api_request(&endpoint)?;
    let resp = builder.send().await?;
    debug!("Response headers: {:#?}\n", resp.headers());
    debug!(
        "Got response: {} {:?}",
        resp.status().as_str(),
        resp.status().canonical_reason()
    );
    Ok(resp)
}

fn build_api_request(endpoint: &str) -> Result<reqwest::RequestBuilder, RequestError> {
    let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
    let apikey = config::read_api_key()?;
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());
    let builder = build_request(&url).headers(headers);
    Ok(builder)
}

fn build_request(url: &Url) -> reqwest::RequestBuilder {
    debug!("Building request to: {}", &url);
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    let client = reqwest::Client::new();
    let builder = client.get(url.clone()).headers(headers);
    builder
}
