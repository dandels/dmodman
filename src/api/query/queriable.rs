use crate::api::error::RequestError;
use crate::api::Client;
use crate::util::format;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait Queriable: DeserializeOwned {
    const FORMAT_STRING: &'static str;

    async fn request(client: &Client, params: Vec<&str>) -> Result<Self, RequestError> {
        let endpoint = format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        client.request_counter.clone().push(&resp.headers());
        let ret: Self = serde_json::from_value(resp.json().await?).unwrap();
        Ok(ret)
    }
}
