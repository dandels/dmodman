use super::{Cacheable, FileList, NxmUrl, Requestable};
use crate::db::LocalFile;
use crate::lookup;
use super::search::*;
use super::error::RequestError;
use crate::{config, utils};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Response;
use std::io::Write;
use std::path::{Path, PathBuf};
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

    // create metadata json file
    let lf = LocalFile::new(&nxm, file_name);
    lf.write()?;

    // TODO: should just do an Md5Search instead? It would allows us to validate the file while getting its metadata

    let file_list_needs_refresh: bool;
    match lookup::file_list(&nxm.domain_name, &nxm.mod_id).await {
        // need to redownload file list if the cached one doesn't have file with this file_id
        Ok(fl) => { file_list_needs_refresh = fl.files.iter().find(|fd| fd.file_id == nxm.file_id).is_none(); },
        Err(_) => { file_list_needs_refresh = true; }
    }
    if file_list_needs_refresh {
        let fl = FileList::request(vec![&nxm.domain_name, &nxm.mod_id.to_string()]).await?;
        fl.save_to_cache(&nxm.domain_name, &nxm.mod_id)?;
    }

    Ok(path)
}

async fn download_buffered(url: &Url, path: &Path) -> Result<(), RequestError> {
    let mut buffer = std::fs::File::create(path)?;
    println!("downloading to... {:?}", path.as_os_str());
    let builder = build_request(&url);
    let resp: reqwest::Response = builder.send().await?;
    buffer.write_all(&resp.bytes().await?)?;
    println!("download complete");
    Ok(())
}

pub async fn send_api_request(endpoint: &str) -> Result<Response, RequestError> {
    let builder = build_api_request(&endpoint)?;
    let resp = builder.send().await?;
    /* Enable this to see response headers.
     * TODO the response contains a count of remaining API request quota and would be useful to track
    //println!("Response headers: {:#?}\n", resp.headers());
    //println!(
    //    "Got response: {} {:?}",
    //    resp.status().as_str(),
    //    resp.status().canonical_reason()
    //);
    */
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
    //println!("Building request to: {}", &url);
    let mut headers = HeaderMap::new();
    let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();
    headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());
    let client = reqwest::Client::new();

    client.get(url.clone()).headers(headers)
}
