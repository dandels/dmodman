/* The NXM link format isn't part of the API, but included here for convenience.
 */

use crate::api::error::NxmDownloadError;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub struct NxmUrl {
    pub url: Url,
    pub query: String,
    pub domain_name: String, // this is the game name
    pub mod_id: u32,
    pub file_id: u64,
    pub key: String,
    pub expires: u128,
    pub user_id: u32,
}

impl FromStr for NxmUrl {
    type Err = NxmDownloadError;

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
        let expires: u128 = query_pairs.next().unwrap().1.parse()?;
        check_expiration(&expires)?;
        let user_id: u32 = query_pairs.next().unwrap().1.parse()?;

        Ok(NxmUrl {
            url: url,
            query: query,
            domain_name: check_game_special_case(game),
            mod_id: mod_id,
            file_id: file_id,
            key: key,
            expires: expires,
            user_id: user_id,
        })
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

fn check_expiration(time: &u128) -> Result<(), NxmDownloadError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    match time > &now {
        true => Ok(()),
        false => Err(NxmDownloadError::Expired),
    }
}
