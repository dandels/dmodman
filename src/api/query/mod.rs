mod download_link;
mod file_list;
mod games;
mod md5_search;
mod mod_info;
mod search;
mod updated;

pub use self::download_link::*;
pub use self::file_list::*;
#[allow(unused_imports)]
pub use self::games::*; // unused endpoint
pub use self::md5_search::*;
pub use self::mod_info::*;
pub use self::search::*;
pub use self::updated::*;

use crate::api::ApiError;
use crate::api::Client;
use crate::util::format;
use serde::de::DeserializeOwned;

pub trait Queriable: DeserializeOwned {
    const FORMAT_STRING: &'static str;

    /* TODO don't crash if server returns unexpected response, log response instead.
     * Currently unimplemented because the UI is unable to wrap long messages. */
    async fn request(client: &Client, params: &[&str]) -> Result<Self, ApiError> {
        let endpoint = format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        client.request_counter.push(resp.headers()).await;

        Ok(serde_json::from_value::<Self>(resp.json().await?)?)
    }
}
