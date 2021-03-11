use crate::config;
use std::path::PathBuf;

pub enum PathType<'a> {
    DownloadLink(&'a str, &'a u32, &'a u64), // game, mod_id, file_id
    FileDetails(&'a str, &'a u32, &'a u64),  // game, mod_id, file_id
    FileList(&'a str, &'a u32),              // game, mod_id
    GameInfo(&'a str),                       // game
    Md5Search(&'a str, &'a u32, &'a u64),    // game, mod_id, file_id
    ModInfo(&'a str, &'a u32),               // game, mod_id
}

impl PathType<'_> {
    pub fn path(&self) -> PathBuf {
        const DL_LINKS: &str = "download_links";
        const FILE_DETAILS: &str = "file_details";
        const FILE_LISTS: &str = "file_lists";
        const MD5_SEARCH: &str = "md5_search";
        const MOD_INFO: &str = "mod_info";

        let mut path;

        match self {
            Self::DownloadLink(game, mod_id, file_id) => {
                path = config::cache_dir(&game);
                path.push(DL_LINKS);
                path.push(format!("{}-{}.json", mod_id.to_string(), file_id.to_string()));
            }
            Self::FileDetails(game, mod_id, file_id) => {
                path = config::cache_dir(&game);
                path.push(FILE_DETAILS);
                path.push(format!("{}-{}.json", mod_id.to_string(), file_id.to_string()));
            }
            Self::FileList(game, mod_id) => {
                path = config::cache_dir(&game);
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id.to_string()));
            }
            Self::GameInfo(game) => {
                path = config::cache_dir(&game);
                path.push(format!("{}.json", game.to_string()));
            }
            Self::Md5Search(game, mod_id, file_id) => {
                path = config::cache_dir(&game);
                path.push(MD5_SEARCH);
                path.push(format!("{}-{}.json", mod_id.to_string(), file_id.to_string()));
            }
            Self::ModInfo(game, mod_id) => {
                path = config::cache_dir(&game);
                path.push(MOD_INFO);
                path.push(format!("{}.json", mod_id.to_string()));
            }
        }
        path
    }
}
