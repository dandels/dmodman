use log::debug;
use md5::{Digest, Md5};
use std::fs::File;
use std::path::PathBuf;
use url::Url;

pub fn file_name_from_url(url: &Url) -> String {
    let path_segments = url.path_segments().unwrap();
    let encoded = path_segments.last().unwrap();
    let decode = percent_encoding::percent_decode(encoded.as_bytes());
    let file_name = decode.decode_utf8_lossy().to_string();
    file_name
}

pub fn md5sum(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    //let mut reader = BufReader::new(file);
    let mut hasher = Md5::new();
    let bytes_read = std::io::copy(&mut file, &mut hasher)?;
    debug!("Bytes read: {}", bytes_read);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

pub fn mkdir_recursive(path: &PathBuf) {
    std::fs::create_dir_all(path.clone().to_str().unwrap()).expect(&format!(
        "Unable to create directory at {}",
        path.to_str().unwrap()
    ));
}
