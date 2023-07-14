use crate::api::ApiError;
use crate::api::Client;
use crate::util::format;
use crate::Messages;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tokio::task;

#[async_trait]
pub trait Queriable: DeserializeOwned {
    const FORMAT_STRING: &'static str;

    /* TODO don't crash if server returns unexpected response, log response instead.
     * Currently unimplemented because the UI is unable to wrap long messages. */
    async fn request(client: &Client, _msgs: Messages, params: Vec<&str>) -> Result<Self, ApiError> {
        let endpoint = format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        client.request_counter.push(resp.headers()).await;

        task::spawn_blocking(
            move || async move { Ok(serde_json::from_value::<Self>(resp.json().await.unwrap()).unwrap()) },
        )
        .await
        .unwrap()
        .await
    }
}
