use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{BufReader, Error, Write};
use std::path::PathBuf;
use std::{fs, fs::File};

pub trait Cacheable: Serialize + DeserializeOwned + Send
where
    Self: 'static,
{
    async fn save(&self, path: PathBuf) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)?;
        tokio::task::spawn_blocking(move || {
            fs::create_dir_all(path.parent().unwrap().to_str().unwrap())?;
            let mut file = File::create(path)?;
            file.write_all(data.as_bytes())?;
            Ok(())
        })
        .await
        .unwrap()
    }

    async fn save_compressed(&self, path: PathBuf) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)?;
        tokio::task::spawn_blocking(move || {
            fs::create_dir_all(path.parent().unwrap().to_str().unwrap())?;
            let file = File::create(path.with_extension("json.zst"))?;
            let mut encoder = zstd::Encoder::new(file, 0)?;
            encoder.write_all(data.as_bytes())?;
            encoder.finish()?;
            Ok(())
        })
        .await
        .unwrap()
    }

    async fn load(path: PathBuf) -> Result<Self, Error> {
        tokio::task::spawn_blocking(move || {
            if let Ok(zst_file) = File::open(path.with_extension("json.zst")) {
                let decoder = zstd::Decoder::new(zst_file)?;
                let mut reader = BufReader::new(decoder);
                Ok(serde_json::from_reader(&mut reader)?)
            } else {
                Ok(serde_json::from_str(&fs::read_to_string(&path)?)?)
            }
        })
        .await
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Cacheable;
    use crate::api::{ApiError, FileList, ModInfo};
    use crate::config::{ConfigBuilder, DataType};
    use crate::Logger;

    #[tokio::test]
    async fn read_cached_mod_info() -> Result<(), ApiError> {
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::load(Logger::default()).unwrap().profile(game).build().unwrap();
        let path = config.path_for(DataType::ModInfo(game, mod_id));
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
        let path = config.path_for(DataType::FileList(game, mod_id));

        let fl = FileList::load(path).await?;
        let mut upds = fl.file_updates.clone();
        while let Some(upd) = upds.pop() {
            println!("current : {}", upd.uploaded_timestamp);
        }
        assert_eq!(1000014198, fl.files.first().unwrap().id.0);
        assert_eq!(fl.files.first().unwrap().name, "Graphic Herbalism MWSE");
        assert_eq!(fl.file_updates.last().unwrap().old_file_name, "GH TR - PT Meshes-46599-1-01-1556986716.7z");
        Ok(())
    }
}
