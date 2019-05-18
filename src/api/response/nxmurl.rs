use std::time::{SystemTime, UNIX_EPOCH};
use url::{ParseError, Url};

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

impl NxmUrl {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        return self.expires < now;
    }

    pub fn parse(url: &str) -> Result<NxmUrl, ParseError> {
        let url = Url::parse(&url)?;

        let mut path_segments = url.path_segments().unwrap();
        let game = url.host().unwrap().to_string();
        let _mods = path_segments.next();
        let mod_id: u32 = path_segments.next().unwrap().parse().unwrap();
        let _files = path_segments.next();
        let file_id: u64 = path_segments.next().unwrap().parse().unwrap();
        let q = url.clone();
        let query = q.query().unwrap();
        let mut query_pairs = url.query_pairs();
        let key: String = query_pairs.next().unwrap().1.to_string();
        let expires: u128 = query_pairs.next().unwrap().1.parse().unwrap();
        let user_id: u32 = query_pairs.next().unwrap().1.parse().unwrap();

        let nxm = NxmUrl {
            url: url,
            query: query.to_string(),
            domain_name: check_game_special_case(game),
            mod_id: mod_id,
            file_id: file_id,
            key: key,
            expires: expires,
            user_id: user_id,
        };
        Ok(nxm)
    }
}

/* The nxm link protocol isn't synced with the API protocol for all games. At least these two are
 * special cases, but there might be more. */
pub fn check_game_special_case(game: String) -> String {
    let g = game.to_ascii_lowercase();
    match g.as_str() {
        "skyrimse" => "skyrimspecialedition".to_string(),
        "falloutnv" => "newvegas".to_string(),
        &_ => g,
    }
}
