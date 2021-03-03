use async_trait::async_trait;
use crate::db::Cacheable;
use crate::api::error::RequestError;
use crate::utils;
use crate::api::Client;

#[async_trait]
pub trait Queriable: Cacheable {
    const FORMAT_STRING: &'static str;

    async fn request(client: &Client, params: Vec<&str>) -> Result<Self, RequestError> {
        let endpoint = utils::format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        let ret: Self = serde_json::from_value(resp.json().await?).unwrap();
        Ok(ret)
    }
}
