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
    DownloadLink(&'a u32, &'a u64), // game, mod_id, file_id
    FileList(&'a u32),               // game, mod_id
    GameInfo(),                      // game
    Md5Search(&'a u32, &'a u64),     // game, mod_id, file_id
    ModInfo(&'a u32),                // game, mod_id

    // Local formats
    LocalFile(&'a LocalFile),
}

impl Config {
    pub fn path_for(&self, path_type: PathType) -> PathBuf {
        let mut path;

        match path_type {
            PathType::DownloadLink(mod_id, file_id) => {
                path = self.game_cache_dir();
                path.push(DL_LINKS);
                path.push(format!("{}-{}.json", mod_id.to_string(), file_id.to_string()));
            }
            PathType::FileList(mod_id) => {
                path = self.game_cache_dir();
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id.to_string()));
            }
            PathType::GameInfo() => {
                path = self.game_cache_dir();
                path.push(format!("{}.json", self.game));
            }
            PathType::Md5Search(mod_id, file_id) => {
                path = self.game_cache_dir();
                path.push(MD5_SEARCH);
                path.push(format!("{}-{}.json", mod_id.to_string(), file_id.to_string()));
            }
            PathType::ModInfo(mod_id) => {
                path = self.game_cache_dir();
                path.push(MOD_INFO);
                path.push(format!("{}.json", mod_id.to_string()));
            }
            PathType::LocalFile(lf) => {
                path = self.download_dir();
                path.push(format!("{}.json", lf.file_name));
            }
        }
        path
    }
}
