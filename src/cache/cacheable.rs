use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{BufReader, Error, Write};
use std::path::PathBuf;
use std::{fs, fs::File};

pub trait Cacheable: Serialize + DeserializeOwned + Send
where
    Self: 'static,
{
    async fn save<T: Into<PathBuf>>(&self, path: T) -> Result<(), Error> {
        let path = path.into();
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

    async fn save_compressed<T: Into<PathBuf>>(&self, path: T) -> Result<(), Error> {
        let path = path.into();
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

    async fn load<T: Into<PathBuf>>(path: T) -> Result<Self, Error> {
        let path = path.into();
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
    use crate::config::{ConfigBuilder, DataPath};
    use crate::Logger;
    use std::path::PathBuf;

    #[tokio::test]
    async fn read_cached_mod_info() -> Result<(), ApiError> {
        let profile = "testprofile";
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::load(Logger::default()).unwrap().profile(profile).build().unwrap();
        let path: PathBuf = DataPath::ModInfo(&config, game, mod_id).into();
        println!("{:?}", path);

        let mi: ModInfo = ModInfo::load(path).await?;
        assert_eq!(mi.name.unwrap(), "Graphic Herbalism - MWSE and OpenMW Edition");
        Ok(())
    }

    #[tokio::test]
    async fn read_cached_file_list() -> Result<(), ApiError> {
        let profile = "testprofile";
        let game = "morrowind";
        let mod_id = 46599;

        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let path = DataPath::FileList(&config, game, mod_id);

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
