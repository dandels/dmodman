use crate::db::{Cache, LocalFile};
use super::query::{FileList, Search, Queriable};
use super::NxmUrl;
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

const API_URL: &str = "https://api.nexusmods.com/v1/";
const SEARCH_URL: &str = "https://search.nexusmods.com/mods";

pub struct Client {
    cache: &'static mut Cache,
    client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    api_headers: reqwest::header::HeaderMap,
}

impl Client {
    pub fn new(cache: &'static mut Cache) -> Result<Self, RequestError> {
        let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());

        let apikey = config::read_api_key()?;
        let mut api_headers = headers.clone();
        api_headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());

        let client = reqwest::Client::new();
        Ok(Self { cache , client, headers, api_headers})
    }


    fn build_request(&self, url: Url) -> reqwest::RequestBuilder {
        self.client.get(url).headers(self.headers.clone())
    }

    fn build_api_request(&self, endpoint: &str) -> Result<reqwest::RequestBuilder, RequestError> {
        let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
        Ok(self.client.get(url).headers(self.api_headers.clone()))
    }

    pub async fn send_api_request(&self, endpoint: &str) -> Result<Response, RequestError> {
        let builder = self.build_api_request(&endpoint)?;
        let resp = builder.send().await?;
        /* TODO the response headers contain a count of remaining API request quota and would be useful to track
         * println!("Response headers: {:#?}\n", resp.headers());
         * println!(
         *     "Got response: {} {:?}",
         *     resp.status().as_str(),
         *     resp.status().canonical_reason()
         * );
        */
        Ok(resp)
    }

    async fn download_buffered(&self, url: Url, path: &Path) -> Result<(), RequestError> {
        let mut buffer = std::fs::File::create(path)?;
        let builder = self.build_request(url);
        let resp: reqwest::Response = builder.send().await?;
        buffer.write_all(&resp.bytes().await?)?;
        Ok(())
    }

    // TODO test this
    pub async fn mod_search(&self, query: String) -> Result<Search, RequestError> {
        let base: Url = Url::parse(SEARCH_URL).unwrap();
        let url = base.join(&query).unwrap();
        let builder = self.build_request(url);
        Ok(builder.send().await?.json().await?)
    }

    pub async fn download_mod_file(&mut self, nxm: &NxmUrl, url: Url) -> Result<PathBuf, RequestError> {
        let file_name = utils::file_name_from_url(&url);
        let mut path = config::download_dir(&nxm.domain_name);
        std::fs::create_dir_all(path.clone().to_str().unwrap())?;
        path.push(&file_name.to_string());

        self.download_buffered(url, &path).await?;

        // create metadata json file
        let lf = LocalFile::new(&nxm, file_name);
        lf.write()?;

        /* TODO: should just do an Md5Search instead? It would allows us to validate the file while getting its metadata
         * However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312
         */

        if self.cache.file_details_map.get(&nxm.file_id).is_none() {
            let fl = FileList::request(&self, vec![&nxm.domain_name, &nxm.mod_id.to_string()]).await?;
            self.cache.save_file_list(fl, &nxm.mod_id)?;
        }
        Ok(path)
    }
}
