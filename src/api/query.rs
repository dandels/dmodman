pub use super::downloads::nxm_url::*;
use super::nexus_api::*;
use crate::api::ApiError;
use crate::cache::ModFileMetadata;
use crate::util;
use crate::{Cache, Client, Config, Logger};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use url::Url;

const SEARCH_URL: &str = "https://search.nexusmods.com/mods";

#[derive(Clone)]
pub struct Query {
    cache: Cache,
    client: Client,
    #[allow(dead_code)]
    config: Arc<Config>,
    logger: Logger,
}

impl Query {
    pub fn new(cache: Cache, client: Client, config: Arc<Config>, logger: Logger) -> Self {
        Self {
            cache,
            client,
            config,
            logger,
        }
    }

    pub async fn verify_metadata(&self, mfd: Arc<ModFileMetadata>) {
        if mfd.file_details().await.is_none() {
            let _ = self.file_list(&mfd.game, mfd.mod_id).await;
        }
        if mfd.mod_info.read().await.is_none() {
            let _ = self.mod_info(&mfd.game, mfd.mod_id).await;
        }
    }

    pub async fn download_link(&self, nxm: &NxmUrl) -> Result<Url, ApiError> {
        match DownloadLink::request(
            &self.client,
            // TODO get rid of passing an array as argument
            &[
                &nxm.domain_name,
                &nxm.mod_id.to_string(),
                &nxm.file_id.to_string(),
                &nxm.query,
            ],
        )
        .await
        {
            Ok(dl_links) => {
                self.cache.save_download_links(&dl_links, &nxm.domain_name, nxm.mod_id, nxm.file_id).await?;
                /* The API returns multiple locations for Premium users. The first option is by default the Premium-only
                 * global CDN, unless the user has selected a preferred download location.
                 * For small files the download URL is the same regardless of location choice.
                 * Free-tier users only get one location choice.
                 * Anyway, we can just pick the first location. */
                let location = dl_links.locations.first().unwrap();
                match Url::parse(&location.URI) {
                    Ok(url) => Ok(url),
                    Err(e) => {
                        self.logger.log(format!(
                            "Failed to parse URI in response from Nexus: {}. Please file a bug report.",
                            &location.URI
                        ));
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                self.logger.log(format!("Failed to query download links from Nexus: {}", e));
                Err(e)
            }
        }
    }

    pub async fn mod_info(&self, game: &str, mod_id: u32) -> Result<Arc<ModInfo>, ApiError> {
        let mod_info = Arc::new(ModInfo::request(&self.client, &[game, &mod_id.to_string()]).await?);
        self.cache.save_modinfo(mod_info.clone()).await;
        Ok(mod_info)
    }

    pub async fn file_list(&self, game: &str, mod_id: u32) -> Result<Arc<FileList>, ApiError> {
        let file_list = FileList::request(&self.client, &[game, &mod_id.to_string()]).await?;
        Ok(self.cache.save_file_list(file_list, game, mod_id).await)
    }

    /* Searches for a file matching this
     */
    pub async fn md5search(&self, game: &str, md5: &str, file_name: &str, file_id: u64) -> Result<Md5Result, ApiError> {
        match Md5Search::request(&self.client, &[game, md5]).await {
            Ok(query_res) => match query_res.results.iter().find(|fd| fd.file_details.file_id == file_id) {
                Some(md5result) => {
                    // hash OK
                    if md5.eq(&md5result.file_details.md5) && file_name.eq(&md5result.file_details.file_name) {
                        /* Only store the mod info because the other part of the result is near identical to File
                         * Details. We might be interested in the "Md5FileDetails" in case the other one is somewhy
                         * missing, but dealing with just one kind is a lot simpler.
                         */
                        self.cache.save_modinfo(md5result.mod_info.clone()).await;
                        Ok(md5result.clone())
                    } else {
                        self.logger.log(format!(
                            "Warning: API returned unexpected response when checking hash for {}",
                            &file_name
                        ));
                        let mi = &md5result.mod_info;
                        let fd = &md5result.file_details;
                        self.logger.log(format!("Found {:?}: {} ({})", mi.name, fd.name, fd.file_name));
                        Err(ApiError::HashMismatch)
                    }
                }
                None => {
                    self.logger.log(format!("Failed to verify hash for {}. Found this instead:", file_name));
                    for res in &query_res.results {
                        let mi = &res.mod_info;
                        let fd = &res.file_details;
                        self.logger.log(format!("\t{:?}: {} ({})", mi.name, fd.name, fd.file_name));
                    }
                    Err(ApiError::HashMismatch)
                }
            },
            Err(e) => {
                self.logger.log(format!("Unable to verify integrity of {}: {e}", &file_name));
                self.logger.log("This could mean the download got corrupted.");
                Err(e)
            }
        }
    }

    /* This is unused but should work. Most API requests are easy to implement with serde & traits, but this lacks UI
     * and a sufficiently compelling use case.
     * For example, premium users could search and install mods directly through this application.
     * (Others would have to visit the Nexus, as only premium users can generate download URLs without getting a nxm://
     * URL from the website.) */
    #[allow(dead_code)]
    pub async fn mod_search(&self, query: String) -> Result<Search, ApiError> {
        let base: Url = Url::parse(SEARCH_URL).unwrap();
        let url = base.join(&query).unwrap();
        let builder = self.client.build_request(url)?;
        Ok(builder.send().await?.json().await?)
    }
}

pub trait Queriable: DeserializeOwned {
    const FORMAT_STRING: &'static str;

    /* TODO don't crash if server returns unexpected response, log response instead.
     * Currently unimplemented because the UI is unable to wrap long messages. */
    async fn request(client: &Client, params: &[&str]) -> Result<Self, ApiError> {
        let endpoint = util::format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        client.request_counter.push(resp.headers()).await;

        Ok(serde_json::from_value::<Self>(resp.json().await?)?)
    }
}
#[cfg(test)]
mod tests {
    use crate::api::{ApiError, Client, Query};
    use crate::cache::Cache;
    use crate::ConfigBuilder;
    use crate::Logger;
    use std::sync::Arc;

    #[tokio::test]
    async fn block_test_request() -> Result<(), ApiError> {
        let config = Arc::new(ConfigBuilder::default().build().unwrap());

        let logger = Logger::default();
        let cache = Cache::new(config.clone(), logger.clone()).await.unwrap();
        let client = Client::new(&config).await;
        let query = Query::new(cache.clone(), client.clone(), config.clone(), logger.clone());

        let game = "morrowind";
        let mod_id = 46599;
        match query.file_list(game, mod_id).await {
            Ok(_fl) => panic!("Refresh should have failed"),
            Err(e) => match e {
                ApiError::IsUnitTest => Ok(()),
                _ => {
                    panic!("Refresh should return ApiError::IsUnitTest");
                }
            },
        }
    }
}
