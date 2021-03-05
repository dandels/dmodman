use md5::{Digest, Md5};
use std::fs::File;
use std::path::Path;
use url::Url;

pub fn file_name_from_url(url: &Url) -> String {
    let path_segments = url.path_segments().unwrap();
    let encoded = path_segments.last().unwrap();
    let decode = percent_encoding::percent_decode(encoded.as_bytes());
    let file_name = decode.decode_utf8_lossy().to_string();
    file_name
}

/* The API doesn't offer other hash formats than md5.
 * TODO this implementation is probably not suitable for big files.
 */
pub fn md5sum(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let _bytes_read = std::io::copy(&mut file, &mut hasher)?;
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

//
pub fn human_readable(bytes: u64) -> String {
    let mut bytes: f64 = bytes as f64;
    let units = vec!["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let mut i = 0;
    while (bytes * 10.0).round() / 10.0 >= 1024.0 && i < units.len() - 1 {
        bytes /= 1024.0;
        i += 1;
    }
    if i == 0 {
        return format!("{} {}", bytes as u64, units[i]);
    }
    format!("{:.*} {}", 1, bytes, units[i])
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

    #[test]
    fn human_readable() {
        assert_eq!("272 B", utils::human_readable(272));
        assert_eq!("83.4 KiB", utils::human_readable(85417));
        assert_eq!("204.1 MiB", utils::human_readable(214022328));
        assert_eq!("936.7 MiB", utils::human_readable(982232812));
        assert_eq!("19.9 GiB", utils::human_readable(21402232812));
    }
}
