use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncWriteExt, Error};
use tokio::{fs, fs::File};

use std::path::PathBuf;

pub trait Cacheable: Serialize + DeserializeOwned {
    async fn save(&self, path: PathBuf) -> Result<(), Error> {
        fs::create_dir_all(path.parent().unwrap().to_str().unwrap()).await?;
        let data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(&path).await?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }

    async fn load(path: PathBuf) -> Result<Self, Error> {
        tokio::task::spawn_blocking(move || async move { Ok(serde_json::from_str(&fs::read_to_string(&path).await?)?) })
            .await
            .unwrap()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::Cacheable;
    use crate::api::{ApiError, FileList, ModInfo};
    use crate::config::ConfigBuilder;
    use crate::config::PathType;

    #[tokio::test]
    async fn read_cached_mod_info() -> Result<(), ApiError> {
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::load().unwrap().profile(game).build().unwrap();
        let path = config.path_for(PathType::ModInfo(game, &mod_id));
        println!("{:?}", path);

        let mi: ModInfo = ModInfo::load(path).await?;
        assert_eq!(mi.name.unwrap(), "Graphic Herbalism - MWSE and OpenMW Edition");
        Ok(())
    }

    #[tokio::test]
    async fn read_cached_file_list() -> Result<(), ApiError> {
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::default().profile(game).build().unwrap();
        let path = config.path_for(PathType::FileList(game, &mod_id));

        let fl = FileList::load(path).await?;
        let mut upds = fl.file_updates.clone();
        while let Some(upd) = upds.pop() {
            println!("current : {}", upd.uploaded_timestamp);
        }
        assert_eq!(1000014198, fl.files.first().unwrap().id.0);
        assert_eq!(fl.files.first().unwrap().name, "Graphic Herbalism MWSE");
        assert_eq!(fl.file_updates.peek().unwrap().old_file_name, "GH TR - PT Meshes-46599-1-01-1556986716.7z");
        Ok(())
    }
}
