use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadLocation {
    /* Keys are:
     * "name", "short_name", "URI"
     */
    pub location: Map<String, Value>,
}
