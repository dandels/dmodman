use super::{request, DownloadLocation};
use url::Url;

pub fn handle_nxm_url(nxm_url: &str) -> Result<DownloadLocation, reqwest::Error> {
    let url = Url::parse(&nxm_url).expect("Invalid url.");
    let mut path_segments = url.path_segments().unwrap();
    let mut game: String = url.host().unwrap().to_string();
    game.make_ascii_lowercase();
    let _mods = path_segments.next().unwrap();
    let mod_id: u32 = path_segments.next().unwrap().parse().unwrap();
    let _files = path_segments.next().unwrap();
    let file_id: u64 = path_segments.next().unwrap().parse().unwrap();
    let query = url.query().unwrap();

    let res = request::download_mod_file(&game, &mod_id, &file_id, &query);
    res
}
