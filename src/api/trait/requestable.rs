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
        println!("{}", endpoint);
        let resp = request::send_api_request(&endpoint).await?.error_for_status()?;
        println!("Got api response");
        let val: serde_json::Value = resp.json().await?;
        println!("val: {}", val);
        let ret: Self = serde_json::from_value(val).unwrap();
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::api::r#trait::Cacheable;
    use crate::api::FileList;
    use crate::api::r#trait::requestable::Requestable;

    /* TODO prevent running this as part of normal test suite
     * Making web requests as part of unit testing is not desirable, and the cached version and
     * server version can mismatch at any time.
     */
    #[tokio::test]
    async fn request_file_list() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;
        let cached = FileList::try_from_cache(&game, &mod_id)?;

        let requested: FileList;
        match FileList::request(vec![&game, &mod_id.to_string()]).await {
            Ok(v) => requested = v,
            Err(e) => panic!("{}", e)
        }

        assert_eq!(cached, requested);
        Ok(())
    }
}
