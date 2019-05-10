use super::cmdline;
use super::config;
use super::mod_info::ModInfo;
use std::fs::File;
use std::fs::Metadata;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn save_mod_info(mod_info: &ModInfo) -> Result<(), std::io::Error> {
    let mi = &mod_info;
    let cache_file = config::get_cache_dir();
    let file_name = String::from(mi.mod_id.to_string()) + ".json";
    let mut path = PathBuf::from(cache_file);
    path.push(&mi.domain_name);
    create_dir_if_not_exist(&path);
    path.push(file_name);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(mi)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_mod_info(mod_id: &u32) -> Option<ModInfo> {
    let f = &format!("{}.json", mod_id);
    let mut path = config::get_cache_dir();
    path.push(cmdline::get_game());
    path.push(f);
    let opt_contents = file_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Got mod info from cache. Deserializing json...");
            //Crash if we can't parse the json
            let mi: ModInfo = serde_json::from_str(&v).unwrap();
            return Some(mi)
        }
        Err(_) => {
            println!("Unable to find mod info in cache.");
            return None
        }
    }
}

pub fn file_to_string(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut r = File::open(path)?;
    let mut contents: String = String::new();
    r.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

pub fn create_dir_if_not_exist(path: &PathBuf) {
    let opt_md = path.metadata();
    let md: Metadata;
    match opt_md {
        Ok(v) => md = v.to_owned(),
        Err(_v) => {
            std::fs::create_dir(path.to_str().unwrap()).expect(
                &format!("Unable to create directory at {}", path.to_str().unwrap()));
            md = path.metadata().unwrap();
        }
    }
    assert!(md.is_dir());
}
