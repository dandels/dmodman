pub mod format;

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
 * This is currently unused because the Nexus md5 lookup is broken, see:
 * https://github.com/Nexus-Mods/web-issues/issues/1312
 * TODO use it anyway and report if the hash is wrong.
 *
 * TODO this has to use tokio for reading the file, or it will block the thread
 */
#[allow(dead_code)]
pub fn md5sum(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let _bytes_read = std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

pub fn trim_newline(mut string: String) -> String {
    // We're probably only going to run into Unix line endings, but let's deal with both cases to be sure
    if string.ends_with('\n') {
        string.pop();
        if string.ends_with('\r') {
            string.pop();
        }
    }
    string
}
