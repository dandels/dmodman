use crate::api::query::{DownloadLinks, FileDetails, FileList, GameInfo, Md5Search, ModInfo};
use crate::cache::LocalFile;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncWriteExt, Error};
use tokio::{fs, fs::File};

use std::path::PathBuf;

#[async_trait]
pub trait Cacheable: Serialize + DeserializeOwned {
    async fn save(&self, path: PathBuf) -> Result<(), Error> {
        fs::create_dir_all(path.parent().unwrap().to_str().unwrap()).await?;
        let data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(&path).await?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }

    async fn load(path: PathBuf) -> Result<Self, Error> {
        Ok(serde_json::from_str(&fs::read_to_string(&path).await?)?)
    }
}

impl Cacheable for DownloadLinks {}
impl Cacheable for FileDetails {}
impl Cacheable for FileList {}
impl Cacheable for GameInfo {}
impl Cacheable for LocalFile {}
impl Cacheable for Md5Search {}
impl Cacheable for ModInfo {}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::api::{FileList, ModInfo};
    use crate::cache::cacheable::Cacheable;
    use crate::config::ConfigBuilder;
    use crate::config::PathType;

    #[tokio::test]
    async fn read_cached_mod_info() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::load().unwrap().game(game).build().unwrap();
        let path = config.path_for(PathType::ModInfo(&mod_id));
        println!("{:?}", path);

        let mi: ModInfo = ModInfo::load(path).await?;
        assert_eq!(mi.name, "Graphic Herbalism - MWSE and OpenMW Edition");
        Ok(())
    }

    #[tokio::test]
    async fn read_cached_file_list() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::default().game(game).build().unwrap();
        let path = config.path_for(PathType::FileList(&mod_id));

        let fl = FileList::load(path).await?;
        assert_eq!(1000014198, fl.files.first().unwrap().id.0);
        assert_eq!(fl.files.first().unwrap().name, "Graphic Herbalism MWSE");
        assert_eq!(
            fl.file_updates.first().unwrap().old_file_name,
            "Graphic Herbalism MWSE-46599-1-01-1556688167.7z"
        );
        Ok(())
    }
}
