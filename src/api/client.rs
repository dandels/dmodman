use crate::{config, error_list::ErrorList, utils};
use crate::db::{Cache, Cacheable, LocalFile};

use super::query::{DownloadLink, FileList, Search, Queriable};
use super::{Downloads, DownloadStatus, DownloadState, NxmUrl};
use super::error::RequestError;
use super::error::DownloadError;

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Response;
use url::Url;

use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock };
use std::convert::TryInto;
use std::str::FromStr;

/* API reference:
 * https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0
 */

const API_URL: &str = "https://api.nexusmods.com/v1/";
#[allow(dead_code)]
const SEARCH_URL: &str = "https://search.nexusmods.com/mods";

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    headers: Arc<HeaderMap>,
    api_headers: Arc<Option<HeaderMap>>,
    errors: ErrorList,
    pub downloads: Downloads
}

impl Client {
    pub fn new(errors: ErrorList) -> Result<Self, RequestError> {
        let version = String::from(clap::crate_name!()) + " " + clap::crate_version!();

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());

        let api_headers = match config::read_api_key() {
            Ok(apikey) => {
                let mut api_headers = headers.clone();
                api_headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());
                Some(api_headers)
            },
            Err(e) => {
                errors.push(e.to_string());
                None
            }
        };

        let client = reqwest::Client::new();
        Ok(Self {
            client,
            headers: Arc::new(headers),
            api_headers: Arc::new(api_headers),
            errors,
            downloads: Downloads::default()
        })
    }


    fn build_request(&self, url: Url) -> reqwest::RequestBuilder {
        self.client.get(url).headers((*self.headers).clone())
    }

    fn build_api_request(&self, endpoint: &str) -> Result<reqwest::RequestBuilder, RequestError> {
        let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
        let api_headers = match &*self.api_headers {
            Some(v) => Ok(v.clone()),
            None => Err(RequestError::ApiKeyMissing),
        }?;

        Ok(self.client.get(url).headers(api_headers))
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

    pub async fn queue_download(&mut self, cache: &mut Cache, nxm_str: &str) -> Result<(), DownloadError> {
        let nxm = NxmUrl::from_str(&nxm_str).unwrap();
        let dl = DownloadLink::request(&self, vec![&nxm.domain_name, &nxm.mod_id.to_string(), &nxm.file_id.to_string(), &nxm.query]).await?;
        // TODO only for debugging. Besides, it's not using the file id as it should.
        dl.save_to_cache(&nxm.domain_name, &nxm.mod_id)?;
        let url: Url = Url::parse(&dl.location.URI)?;
        let _file = self.download_mod_file(cache, &nxm, url).await?;
        Ok(())
    }

    async fn download_buffered(&mut self, url: Url, path: &Path, file_name: String, file_id: u64) -> Result<(), DownloadError> {
        let status = Arc::new(RwLock::new(DownloadStatus::new(file_name, file_id)));

        self.downloads.add(status.clone());

        let file = std::fs::File::create(path)?;
        let mut bufwriter = BufWriter::new(&file);
        let builder = self.build_request(url);
        let resp = builder.send().await?;

        status.write().unwrap().bytes_total = resp.content_length();
        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    bufwriter.write_all(&bytes)?;
                    // hope there isn't too much overhead acquiring the lock so often
                    status.write().unwrap().update_progres(bytes.len().try_into().unwrap());
                },
                Err(e) => {
                    status.write().unwrap().state = DownloadState::Failed(DownloadError::from(e));
                }
            }
        }
        bufwriter.flush()?;

        status.write().unwrap().state = DownloadState::Complete;

        Ok(())
    }

    pub async fn download_mod_file(&mut self, cache: &mut Cache, nxm: &NxmUrl, url: Url) -> Result<PathBuf, DownloadError> {
        let file_name = utils::file_name_from_url(&url);
        let mut path = config::download_dir(&nxm.domain_name);
        std::fs::create_dir_all(path.clone().to_str().unwrap())?;
        path.push(&file_name.to_string());

        let lf = LocalFile::new(&nxm, file_name.clone());
        self.download_buffered(url, &path, file_name, nxm.file_id).await?;

        // create metadata json file

        /* TODO: should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata.
         * However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312
         */
        // TODO the cache api needs work
        let file_details_is_cached = cache.save_local_file(lf)?;
        if !file_details_is_cached {
            let fl = FileList::request(&self, vec![&nxm.domain_name, &nxm.mod_id.to_string()]).await?;
            cache.save_file_list(fl, &nxm.mod_id)?;
        }

        Ok(path)
    }

    // TODO test this
    #[allow(dead_code)]
    pub async fn mod_search(&self, query: String) -> Result<Search, RequestError> {
        let base: Url = Url::parse(SEARCH_URL).unwrap();
        let url = base.join(&query).unwrap();
        let builder = self.build_request(url);
        Ok(builder.send().await?.json().await?)
    }
}
