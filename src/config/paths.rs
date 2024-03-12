use std::path::PathBuf;

use super::Config;
use crate::api::downloads::DownloadInfo;

pub const DL_LINKS: &str = "download_links";
pub const FILE_LISTS: &str = "file_lists";
pub const MD5_RESULTS: &str = "md5_results";
pub const MOD_INFO: &str = "mod_info";

#[allow(dead_code)]
pub enum DataPath<'a> {
    // API formats
    DownloadLink(&'a Config, &'a str, u32, u64), // game, mod_id, file_id
    FileList(&'a Config, &'a str, u32),          // game, mod_id
    Md5Results(&'a Config, &'a str, u64),        // game, file_id
    Updated(&'a Config, &'a str),                // game

    // Local formats
    InstalledMod(&'a Config, &'a String),
    ArchiveMetadata(&'a Config, &'a String),
    DownloadInfo(&'a Config, &'a DownloadInfo),

    // Unused API responses
    ModInfo(&'a Config, &'a str, u32), // game, mod_id
    GameInfo(&'a Config, &'a str),     // game

    // Old paths to be checked for backwards compatibility
    FileListCompat(&'a Config, &'a str, u32), // game, mod_id
}

impl From<DataPath<'_>> for PathBuf {
    fn from(value: DataPath) -> Self {
        let mut path;
        match &value {
            DataPath::DownloadInfo(ref config, di) => {
                path = config.download_dir();
                path.push(format!("{}.part.json", di.file_info.file_name));
            }
            DataPath::DownloadLink(ref config, game, mod_id, file_id) => {
                path = config.metadata_for_profile();
                path.push(game);
                path.push(DL_LINKS);
                path.push(format!("{}-{}.json", mod_id, file_id));
            }
            DataPath::FileList(ref config, game, mod_id) => {
                path = config.metadata_for_profile();
                path.push(game);
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id));
            }
            // For version <= 0.2.3
            DataPath::FileListCompat(ref config, game, mod_id) => {
                path = config.data_dir();
                path.push(game);
                path.push(FILE_LISTS);
                path.push(format!("{}.json", mod_id));
            }
            DataPath::GameInfo(ref config, game) => {
                path = config.metadata_dir();
                path.push(format!("{}.json", game));
            }
            DataPath::InstalledMod(ref config, dir_name) => {
                path = config.install_dir();
                path.push(dir_name);
                path.push(".dmodman-meta.json");
            }
            DataPath::ArchiveMetadata(ref config, file_name) => {
                path = config.download_dir();
                path.push(format!("{}.json", file_name));
            }
            DataPath::Md5Results(ref config, game, file_id) => {
                path = config.metadata_dir();
                path.push(game);
                path.push(MD5_RESULTS);
                path.push(format!("{}.json", file_id));
            }
            DataPath::ModInfo(ref config, game, mod_id) => {
                path = config.metadata_dir();
                path.push(game);
                path.push(MOD_INFO);
                path.push(format!("{}.json", mod_id));
            }
            DataPath::Updated(ref config, game) => {
                path = config.data_dir();
                path.push(game);
                path.push("updated.json");
            }
        }
        path
    }
}
