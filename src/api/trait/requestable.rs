use async_trait::async_trait;
use super::Cacheable;
use crate::api::error::RequestError;
use crate::utils;
use crate::api::request;

#[async_trait]
pub trait Requestable: Cacheable {
    const FORMAT_STRING: &'static str;

    async fn request(params: Vec<&str>) -> Result<Self, RequestError> {
        let endpoint = utils::format_string(Self::FORMAT_STRING, params);
        let resp = request::send_api_request(&endpoint).await?.error_for_status()?;
        let ret: Self = resp.json().await?;
        Ok(ret)
    }
}
