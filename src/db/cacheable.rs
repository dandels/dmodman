use crate::config;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;

pub trait Cacheable: Serialize + DeserializeOwned {
    const CACHE_DIR_NAME: &'static str;

    fn cache_dir(game: &str, mod_id: &u32) -> PathBuf {
        let mut path = config::cache_dir(&game);
        path.push(Self::CACHE_DIR_NAME);
        path.push(format!("{}.json", &mod_id.to_string()));
        path
    }

    fn save_to_cache(&self, game: &str, mod_id: &u32) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)?;
        let path = Self::cache_dir(game, mod_id);
        std::fs::create_dir_all(path.parent().unwrap().to_str().unwrap())?;
        println!("creating metadata file: {:?}", path);
        let mut file = File::create(&path)?;
        println!("writing metadata file");
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    // TODO get rid of the mod id here to support more query types
    fn try_from_cache(game: &str, mod_id: &u32) -> Result<Self, Error> {
        let path = Self::cache_dir(game, mod_id);
        let contents = std::fs::read_to_string(path)?;
        let ret = serde_json::from_str(&contents)?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::error::*;
    use crate::api::{FileList, ModInfo};
    use crate::db::Cacheable;
    use crate::test;

    #[test]
    fn read_cached_mod_info() -> Result<(), RequestError> {
        let _rt = test::setup();
        let game = "morrowind";
        let mod_id = 46599;
        let mi: ModInfo = ModInfo::try_from_cache(&game, &mod_id)?;
        assert_eq!(mi.name, "Graphic Herbalism - MWSE and OpenMW Edition");
        Ok(())
    }

    #[test]
    fn read_cached_file_list() -> Result<(), RequestError> {
        let _rt = test::setup();
        let game = "morrowind";
        let mod_id = 46599;
        let fl = FileList::try_from_cache(&game, &mod_id)?;
        assert_eq!(1000014198, fl.files.first().unwrap().id.0);
        assert_eq!(fl.files.first().unwrap().name, "Graphic Herbalism MWSE");
        assert_eq!(
            fl.file_updates.first().unwrap().old_file_name,
            "Graphic Herbalism MWSE-46599-1-01-1556688167.7z"
        );
        Ok(())
    }
}
