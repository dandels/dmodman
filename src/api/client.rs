use crate::cache::{Cache, LocalFile, UpdateStatus};
use crate::{config::Config, util, Messages};

use super::downloads::{DownloadStatus, Downloads, NxmUrl};
use super::error::{DownloadError, RequestError};
use super::query::{DownloadLink, FileList, Queriable, Search};
use super::request_counter::RequestCounter;

use reqwest::header::{HeaderMap, HeaderValue, RANGE, USER_AGENT};
use reqwest::{Response, StatusCode};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;
use url::Url;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

/* API reference:
 * https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0
 */

const API_URL: &str = "https://api.nexusmods.com/v1/";
const SEARCH_URL: &str = "https://search.nexusmods.com/mods";

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    headers: Arc<HeaderMap>,
    api_headers: Arc<Option<HeaderMap>>,
    msgs: Messages,
    cache: Cache,
    config: Config,
    pub downloads: Downloads,
    pub request_counter: RequestCounter,
}

impl Client {
    pub async fn new(cache: &Cache, config: &Config, msgs: &Messages) -> Result<Self, RequestError> {
        let version = String::from(env!("CARGO_CRATE_NAME")) + " " + env!("CARGO_PKG_VERSION");

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());

        let api_headers = match config.apikey.to_owned() {
            Some(apikey) => {
                let mut api_headers = headers.clone();
                // TODO register this app with Nexus so we can get the apikey via SSO login
                api_headers.insert("apikey", HeaderValue::from_str(&apikey).unwrap());
                Some(api_headers)
            }
            None => {
                msgs.push("No apikey configured. API connections are disabled.").await;
                None
            }
        };

        Ok(Self {
            client: reqwest::Client::new(),
            headers: Arc::new(headers),
            api_headers: Arc::new(api_headers),
            msgs: msgs.clone(),
            cache: cache.clone(),
            config: config.clone(),
            downloads: Downloads::default(),
            request_counter: RequestCounter::new(),
        })
    }

    fn build_request(&self, url: Url) -> Result<reqwest::RequestBuilder, RequestError> {
        if cfg!(test) {
            return Err(RequestError::IsUnitTest);
        }
        Ok(self.client.get(url).headers((*self.headers).clone()))
    }

    fn build_api_request(&self, endpoint: &str) -> Result<reqwest::RequestBuilder, RequestError> {
        if cfg!(test) {
            return Err(RequestError::IsUnitTest);
        }
        let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
        let api_headers = match &*self.api_headers {
            Some(v) => Ok(v.clone()),
            None => Err(RequestError::ApiKeyMissing),
        }?;

        Ok(self.client.get(url).headers(api_headers))
    }

    pub async fn send_api_request(&self, endpoint: &str) -> Result<Response, RequestError> {
        let builder = self.build_api_request(endpoint)?;
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

    pub async fn queue_download(&self, nxm_str: String) {
        let me = self.clone();
        let _handle: JoinHandle<Result<(), DownloadError>> = task::spawn(async move {
            let nxm = NxmUrl::from_str(&nxm_str)?;
            let dls = DownloadLink::request(
                &me,
                me.msgs.clone(),
                vec![
                    &nxm.domain_name,
                    &nxm.mod_id.to_string(),
                    &nxm.file_id.to_string(),
                    &nxm.query,
                ],
            )
            .await?;
            me.cache.save_download_links(&dls, &nxm.mod_id, &nxm.file_id).await?;
            /* The API returns multiple locations for Premium users. The first option is by default the Premium-only
             * global CDN, unless the user has selected a preferred download location.
             * For small files the download URL is the same regardless of location choice.
             * Free-tier users only get one location choice.
             * Either way, we can just pick the first location.
             */
            let location = &dls.locations.first().unwrap();
            let url: Url = Url::parse(&location.URI)?;
            let _file = me.download_mod_file(&nxm, url).await?;
            Ok(())
        });
    }

    // TODO look into the threading here, we want downloads to continue when the main thread is blocked
    async fn download_buffered(
        &self,
        url: Url,
        path: &PathBuf,
        file_name: &str,
        file_id: u64,
    ) -> Result<(), DownloadError> {
        self.msgs.push(format!("Downloading to {:?}.", path)).await;
        let mut part_path = path.clone();
        part_path.pop();
        part_path.push(format!("{}.part", file_name));

        let mut builder = self.build_request(url)?;

        /* The HTTP Range header is used to resume downloads.
         * https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range
         */
        let bytes_read = Arc::new(AtomicU64::new(0));
        if part_path.exists() {
            bytes_read.store(std::fs::metadata(&part_path)?.len(), Ordering::Relaxed);
            builder = builder.header(RANGE, format!("bytes={:?}-", bytes_read));
        }

        let resp = builder.send().await?;

        let mut open_opts = OpenOptions::new();
        let mut file;
        match resp.error_for_status_ref() {
            Ok(resp) => {
                file = match resp.status() {
                    StatusCode::OK => open_opts.write(true).create(true).open(&part_path).await?,
                    StatusCode::PARTIAL_CONTENT => open_opts.append(true).open(&part_path).await?,
                    code => panic!("Download {} got unexpected HTTP response: {}", file_name, code),
                };
            }
            Err(e) => {
                self.msgs
                    .push(format!(
                        "Download {} failed with error: {}",
                        file_name,
                        e.status().unwrap()
                    ))
                    .await;
                return Err(DownloadError::from(e));
            }
        }
        let mut status = DownloadStatus::new(file_name.to_string(), file_id, bytes_read, resp.content_length());
        self.downloads.add(status.clone()).await;

        let mut bufwriter = BufWriter::new(&mut file);
        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    bufwriter.write_all(&bytes).await?;
                    status.update_progress(bytes.len() as u64);
                    self.downloads.has_changed.store(true, Ordering::Relaxed);
                }
                Err(e) => {
                    /* The download could fail for network-related reasons. Flush the data we got so that we can
                     * continue it at some later point.
                     */
                    bufwriter.flush().await?;
                    return Err(DownloadError::from(e));
                }
            }
        }
        bufwriter.flush().await?;

        std::fs::rename(part_path, path)?;

        Ok(())
    }

    pub async fn download_mod_file(&self, nxm: &NxmUrl, url: Url) -> Result<PathBuf, DownloadError> {
        let file_name = util::file_name_from_url(&url);
        let mut path = self.config.download_dir();
        std::fs::create_dir_all(path.clone().to_str().unwrap())?;
        path.push(&file_name);

        if path.exists() {
            self.msgs.push(format!("{} already exists and won't be downloaded again.", file_name)).await;
            return Ok(path);
        }

        // check whether download is already in progress
        if self.downloads.get(&nxm.file_id).await.is_some() {
            self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
            return Ok(path);
        }

        self.download_buffered(url, &path, &file_name, nxm.file_id).await?;

        /* TODO: should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata.
         * However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312
         */
        let mut file_index = self.cache.file_index.map.write().await;

        let file_list = match self.cache.file_lists.get((&nxm.domain_name, nxm.mod_id)).await {
            Some(fl) => Some(fl),
            None => {
                match FileList::request(self, self.msgs.clone(), vec![&nxm.domain_name, &nxm.mod_id.to_string()]).await
                {
                    Ok(fl) => {
                        if let Err(e) = self.cache.save_file_list(&fl, &nxm.domain_name, nxm.mod_id).await {
                            self.msgs
                                .push(format!(
                                    "Unable to save file list for {} mod {}: {}",
                                    nxm.domain_name, nxm.mod_id, e
                                ))
                                .await;
                        }
                        Some(fl)
                    }
                    Err(e) => {
                        self.msgs
                            .push(format!(
                                "Unable to query file list for {} mod {}: {}",
                                nxm.domain_name, nxm.mod_id, e
                            ))
                            .await;
                        None
                    }
                }
            }
        };

        let file_details = file_list.and_then(|fl| fl.files.iter().find(|fd| fd.file_id == nxm.file_id).cloned());

        let lf = LocalFile::new(
            nxm,
            file_name,
            UpdateStatus::UpToDate(
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
            ),
        );
        self.cache.add_local_file(lf.clone()).await?;

        file_index.insert(nxm.file_id, (lf, file_details));

        Ok(path)
    }

    // TODO test this
    #[allow(dead_code)]
    pub async fn mod_search(&self, query: String) -> Result<Search, RequestError> {
        let base: Url = Url::parse(SEARCH_URL).unwrap();
        let url = base.join(&query).unwrap();
        let builder = self.build_request(url)?;
        Ok(builder.send().await?.json().await?)
    }
}
