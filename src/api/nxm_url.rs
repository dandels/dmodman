/* The NXM link format isn't part of the API, but included here for convenience.
 */

use crate::api::error::DownloadError;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug)]
pub struct NxmUrl {
    pub url: Url,
    pub query: String,
    pub domain_name: String, // this is the game name
    pub mod_id: u32,
    pub file_id: u64,
    pub key: String,
    pub expires: u64,
    pub user_id: u32,
}

impl FromStr for NxmUrl {
    type Err = DownloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(&s)?;

        let mut path_segments = url.path_segments().unwrap();
        let game = url.host().unwrap().to_string();
        let _mods = path_segments.next();
        let mod_id: u32 = path_segments.next().unwrap().parse()?;
        let _files = path_segments.next();
        let file_id: u64 = path_segments.next().unwrap().parse()?;
        let q = url.clone();
        let query: String = q.query().unwrap().to_string();
        let mut query_pairs = url.query_pairs();
        let key: String = query_pairs.next().unwrap().1.to_string();
        let expires: u64 = query_pairs.next().unwrap().1.parse()?;
        let user_id: u32 = query_pairs.next().unwrap().1.parse()?;

        let ret: NxmUrl = NxmUrl {
            url: url,
            query: query,
            domain_name: check_game_special_case(game),
            mod_id: mod_id,
            file_id: file_id,
            key: key,
            expires: expires,
            user_id: user_id,
        };

        check_expiration(&expires)?;

        Ok(ret)
    }
}

/* The nxm link protocol isn't synced with the API protocol for all games. At least these two are
 * special cases, but there might be more.
 */
fn check_game_special_case(game: String) -> String {
    let g = game.to_ascii_lowercase();
    match g.as_str() {
        "skyrimse" => "skyrimspecialedition".to_string(),
        "falloutnv" => "newvegas".to_string(),
        &_ => g,
    }
}

fn check_expiration(expires: &u64) -> Result<(), DownloadError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    match expires > &now {
        true => Ok(()),
        false => Err(DownloadError::Expired),
    }
}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::api::NxmUrl;
    use std::str::FromStr;

    #[test]
    fn expired_nxm() -> Result<(), DownloadError> {
        let nxm_str = "nxm://SkyrimSE/mods/8850/files/27772?key=XnbXtdAspojLzUAn7x-Grw&expires=1583065790&user_id=1234321";
        if let Err(DownloadError::Expired) = NxmUrl::from_str(&nxm_str) {
            return Ok(());
        }
        panic!("Nxm link should have expired");
    }
}
