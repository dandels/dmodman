use std::fs::File;
use std::fs::Metadata;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn read_to_string(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut r = File::open(path)?;
    let mut contents: String = String::new();
    r.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

// TODO create directories recursively instead of one at a time
pub fn create_dir_if_not_exist(path: &PathBuf) {
    let opt_md = path.metadata();
    let md: Metadata;
    match opt_md {
        Ok(v) => md = v.to_owned(),
        Err(_v) => {
            std::fs::create_dir(path.to_str().unwrap()).expect(&format!(
                "Unable to create directory at {}",
                path.to_str().unwrap()
            ));
            md = path.metadata().unwrap();
        }
    }
    assert!(md.is_dir());
}
