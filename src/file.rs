use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn read_to_string(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut r = File::open(path)?;
    let mut contents: String = String::new();
    r.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

pub fn create_dir_if_not_exist(path: &PathBuf) {
    std::fs::create_dir_all(path.to_str().unwrap()).expect(&format!(
        "Unable to create directory at {}",
        path.to_str().unwrap()
    ));
}
