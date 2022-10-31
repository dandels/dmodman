use crate::api::error::RequestError;
use crate::api::Client;
use crate::util::format;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tokio::task;

#[async_trait]
pub trait Queriable: DeserializeOwned {
    const FORMAT_STRING: &'static str;

    async fn request(client: &Client, params: Vec<&str>) -> Result<Self, RequestError> {
        let endpoint = format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        client.request_counter.clone().push(&resp.headers()).await;

        task::spawn_blocking(
            move || async move { Ok(serde_json::from_value::<Self>(resp.json().await.unwrap()).unwrap()) },
        )
        .await
        .unwrap()
        .await
    }
}
