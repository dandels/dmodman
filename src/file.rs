use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use super::config;
use super::mod_info::ModInfo;

pub fn save_mod_info(mod_info: &ModInfo) {
    let mi = &mod_info;
    let mut cache_file = config::get_cache_dir();
    let file_name = String::from(
        mi.mod_id.to_string()) + ".json";
    cache_file.push(file_name);
    write_mod_info_file(&PathBuf::from(cache_file), mi).unwrap();
}

pub fn write_mod_info_file(path: &PathBuf, mi: &ModInfo) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let data = serde_json::to_string_pretty(mi)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_mod_info(mod_id: &u32) -> Option<ModInfo> {
    let f = &format!("{}.json", mod_id);
    let mut path = config::get_cache_dir();
    path.push(f);
    let opt_contents = file_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Got mod info from cache. Deserializing json...");
            //Crash if we can't parse the json
            let mi: ModInfo = serde_json::from_str(&v).unwrap();
            return Some(mi)
        },
        Err(_) => {
            println!("Unable to read mod info from cache.");
            return None
        }
    }
}

pub fn file_to_string(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut r = File::open(path)?;
    let mut contents: String = String::new();
    r.read_to_string(&mut contents)?;
    Ok(contents)
}
