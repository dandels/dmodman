use crate::api::query::{DownloadLinks, FileDetails, FileList, GameInfo, Md5Search, ModInfo};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncWriteExt, Error};
use tokio::{fs, fs::File};

use std::path::PathBuf;

#[async_trait]
pub trait Cacheable: Serialize + DeserializeOwned {
    async fn save_to_cache(&self, path: PathBuf) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)?;
        fs::create_dir_all(path.parent().unwrap().to_str().unwrap()).await?;
        let mut file = File::create(&path).await?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }

    async fn try_from_cache(path: PathBuf) -> Result<Self, Error> {
        let contents = fs::read_to_string(&path).await?;
        let ret = serde_json::from_str(&contents)?;
        Ok(ret)
    }
}

impl Cacheable for DownloadLinks {}
impl Cacheable for FileDetails {}
impl Cacheable for FileList {}
impl Cacheable for GameInfo {}
impl Cacheable for Md5Search {}
impl Cacheable for ModInfo {}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::api::{FileList, ModInfo};
    use crate::cache::cacheable::Cacheable;
    use crate::cache::PathType;

    #[tokio::test]
    async fn read_cached_mod_info() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;
        let path = PathType::ModInfo(&game, &mod_id).path();
        let mi: ModInfo = ModInfo::try_from_cache(path).await?;
        assert_eq!(mi.name, "Graphic Herbalism - MWSE and OpenMW Edition");
        Ok(())
    }

    #[tokio::test]
    async fn read_cached_file_list() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;
        let path = PathType::FileList(&game, &mod_id).path();
        let fl = FileList::try_from_cache(path).await?;
        assert_eq!(1000014198, fl.files.first().unwrap().id.0);
        assert_eq!(fl.files.first().unwrap().name, "Graphic Herbalism MWSE");
        assert_eq!(
            fl.file_updates.first().unwrap().old_file_name,
            "Graphic Herbalism MWSE-46599-1-01-1556688167.7z"
        );
        Ok(())
    }
}
