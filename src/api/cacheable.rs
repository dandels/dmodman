use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;

pub trait Cacheable: Serialize + DeserializeOwned {
    const CACHE_DIR_NAME: &'static str;

    fn cache_dir(game: &str, mod_id: &u32) -> PathBuf {
        let mut path = dirs::cache_dir().unwrap();
        path.push(Self::CACHE_DIR_NAME);
        path.push(&game);
        path.push(&mod_id.to_string());
        path
    }

    fn save_to_cache(&self, game: &str, mod_id: &u32) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)?;
        let path = Self::cache_dir(&game, &mod_id);
        std::fs::create_dir_all(path.parent().unwrap().to_str().unwrap())?;
        let mut file = File::create(&path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn try_from_cache(game: &str, mod_id: &u32) -> Result<Self, Error> {
        let path = Self::cache_dir(&game, &mod_id);
        let contents = std::fs::read_to_string(path)?;
        let ret = serde_json::from_str(&contents)?;
        Ok(ret)
    }
}
