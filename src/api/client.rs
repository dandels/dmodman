use super::request_counter::RequestCounter;
use super::ApiError;
use crate::config::Config;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Response;
use std::sync::Arc;
use url::Url;

/* API reference:
 * https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0
 */

const API_URL: &str = "https://api.nexusmods.com/v1/";

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    headers: Arc<HeaderMap>,
    api_headers: Arc<Option<HeaderMap>>,
    pub request_counter: RequestCounter,
}

impl Client {
    pub async fn new(config: &Arc<Config>) -> Self {
        let version = String::from(env!("CARGO_CRATE_NAME")) + " " + env!("CARGO_PKG_VERSION");

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&version).unwrap());

        let api_headers = match &config.apikey {
            Some(apikey) => {
                let mut api_headers = headers.clone();
                api_headers.insert("apikey", HeaderValue::from_str(apikey).unwrap());
                Some(api_headers)
            }
            None => None,
        };

        Self {
            client: reqwest::Client::new(),
            headers: Arc::new(headers),
            api_headers: Arc::new(api_headers),
            request_counter: RequestCounter::new(),
        }
    }

    pub fn build_request(&self, url: Url) -> Result<reqwest::RequestBuilder, ApiError> {
        if cfg!(test) {
            return Err(ApiError::IsUnitTest);
        }
        Ok(self.client.get(url).headers((*self.headers).clone()))
    }

    fn build_api_request(&self, endpoint: &str) -> Result<reqwest::RequestBuilder, ApiError> {
        if cfg!(test) {
            return Err(ApiError::IsUnitTest);
        }
        let url: Url = Url::parse(&(String::from(API_URL) + endpoint)).unwrap();
        let api_headers = match &*self.api_headers {
            Some(v) => Ok(v.clone()),
            None => Err(ApiError::ApiKeyMissing),
        }?;

        Ok(self.client.get(url).headers(api_headers))
    }

    pub async fn send_api_request(&self, endpoint: &str) -> Result<Response, ApiError> {
        let builder = self.build_api_request(endpoint)?;
        let resp = builder.send().await?;
        /* The response headers contain a count of remaining API request quota and are tracked in api/query/queriable.rs
         * println!("Response headers: {:#?}\n", resp.headers());
         * println!(
         *     "Got response: {} {:?}",
         *     resp.status().as_str(),
         *     resp.status().canonical_reason()
         * ); */
        Ok(resp)
    }
}
