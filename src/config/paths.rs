use std::path::PathBuf;

use super::Config;
use crate::api::downloads::DownloadInfo;
use crate::cache::ArchiveFile;

pub const DL_LINKS: &str = "download_links";
pub const FILE_LISTS: &str = "file_lists";
pub const MD5_RESULTS: &str = "md5_results";
pub const MOD_INFO: &str = "mod_info";

#[allow(dead_code)]
pub enum DataType<'a> {
    // API formats
    DownloadLink(&'a str, u32, u64), // game, mod_id, file_id
    FileList(&'a str, u32),          // game, mod_id
    Md5Results(&'a str, u64),        // game, file_id
    Updated(&'a str),                // game

    // Local formats
    InstalledMod(&'a String),
    ArchiveMetadata(&'a ArchiveFile),
    DownloadInfo(&'a DownloadInfo),

    // Unused API responses
    ModInfo(&'a str, u32), // game, mod_id
    GameInfo(&'a str),     // game
}

impl Config {
    pub fn path_for(&self, data_type: DataType) -> PathBuf {
        let mut path;

        match data_type {
            DataType::DownloadInfo(di) => {
                path = self.download_dir();
                path.push(format!("{}.part.json", di.file_info.file_name));
            }
            DataType::DownloadLink(game, mod_id, file_id) => {
                path = self.data_dir();
                path.push(game);
                path.push(DL_LINKS);
                path.push(format!("{}-{}.json", mod_id, file_id));
            }
            DataType::FileList(game, mod_id) => {
                path = self.data_dir();
                path.push(game);
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id));
            }
            DataType::GameInfo(game) => {
                path = self.data_dir();
                path.push(format!("{}.json", game));
            }
            DataType::InstalledMod(dir_name) => {
                path = self.install_dir();
                path.push(dir_name);
                path.push(".dmodman-meta.json");
            }
            DataType::ArchiveMetadata(af) => {
                path = self.download_dir();
                path.push(format!("{}.json", af.file_name));
            }
            DataType::Md5Results(game, file_id) => {
                path = self.data_dir();
                path.push(game);
                path.push(MD5_RESULTS);
                path.push(format!("{}.json", file_id));
            }
            DataType::ModInfo(game, mod_id) => {
                path = self.data_dir();
                path.push(game);
                path.push(MOD_INFO);
                path.push(format!("{}.json", mod_id));
            }
            DataType::Updated(game) => {
                path = self.data_dir();
                path.push(game);
                path.push("updated.json");
            }
        }
        path
    }
}
