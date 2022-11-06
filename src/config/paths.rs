use super::Config;

use crate::cache::LocalFile;
use std::path::PathBuf;

pub const DL_LINKS: &str = "download_links";
pub const FILE_LISTS: &str = "file_lists";
pub const MD5_SEARCH: &str = "md5_search";
pub const MOD_INFO: &str = "mod_info";

#[allow(dead_code)]
pub enum PathType<'a> {
    // API formats
    DownloadLink(&'a str, &'a u32, &'a u64), // game, mod_id, file_id
    FileList(&'a str, &'a u32),              // game, mod_id
    GameInfo(&'a str),                       // game
    Md5Search(&'a str, &'a u32, &'a u64),    // game, mod_id, file_id
    ModInfo(&'a str, &'a u32),               // game, mod_id

    // Local formats
    LocalFile(&'a LocalFile),
}

impl Config {
    pub fn path_for(&self, path_type: PathType) -> PathBuf {
        let mut path;

        match path_type {
            PathType::DownloadLink(game, mod_id, file_id) => {
                path = self.cache_dir();
                path.push(game);
                path.push(DL_LINKS);
                path.push(format!("{}-{}.json", mod_id, file_id));
            }
            // The game needs to be specified to support cross-game modding, reading the config doesn't work.
            PathType::FileList(game, mod_id) => {
                path = self.cache_dir();
                path.push(game);
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id));
            }
            PathType::GameInfo(game) => {
                path = self.cache_dir();
                path.push(format!("{}.json", game));
            }
            PathType::Md5Search(game, mod_id, file_id) => {
                path = self.cache_dir();
                path.push(game);
                path.push(MD5_SEARCH);
                path.push(format!("{}-{}.json", mod_id, file_id));
            }
            PathType::ModInfo(game, mod_id) => {
                path = self.cache_dir();
                path.push(game);
                path.push(MOD_INFO);
                path.push(format!("{}.json", mod_id));
            }
            PathType::LocalFile(lf) => {
                path = self.download_dir();
                path.push(format!("{}.json", lf.file_name));
            }
        }
        path
    }
}
