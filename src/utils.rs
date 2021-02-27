use log::debug;
use md5::{Digest, Md5};
use std::fs::File;
use std::path::PathBuf;
use url::Url;

// The last part of the url is the file name
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

pub fn format_string(format_string: &str, params: Vec<&str>) -> String {
    let parts: Vec<&str> = format_string.split("{}").collect();

    let mut ret = String::new();

    for i in 0..parts.len() - 1 {
        ret.push_str(parts[i]);
        ret.push_str(params[i]);
    }
    if let Some(tail) = parts.last() {
        ret.push_str(tail);
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::utils;

    #[test]
    fn endpoint_format() {
        let arg = "games/{}/mods/{}/files.json";
        let params = vec!["morrowind", "46599"];

        assert_eq!(
            "games/morrowind/mods/46599/files.json",
            utils::format_string(&arg, params)
        );
    }
}
