pub mod format;

use md5::{Digest, Md5};
use std::path::PathBuf;
use tokio::task;
use url::Url;

pub fn file_name_from_url(url: &Url) -> String {
    let path_segments = url.path_segments().unwrap();
    let encoded = path_segments.last().unwrap();
    let decode = percent_encoding::percent_decode(encoded.as_bytes());
    let file_name = decode.decode_utf8_lossy().to_string();
    file_name
}

/* The API doesn't offer other hash formats than md5. We could get the sha256 sum via the 3rd party virus scan URL for
 * those files that have it, but that is very clunky. Still, it's an option in case Nexus still has issues with their
 * md5 sums: https://github.com/Nexus-Mods/web-issues/issues/1312
 */
pub async fn md5sum(path: PathBuf) -> Result<String, std::io::Error> {
    task::spawn_blocking(move || {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Md5::new();
        let _bytes_read = std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    })
    .await?
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
