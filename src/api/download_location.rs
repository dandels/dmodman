use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug,Serialize, Deserialize)]
pub struct DownloadLocation {
    pub location: Map<String, Value>
}
