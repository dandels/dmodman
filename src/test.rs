use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;

// Used only by tests, so the compiler warns about dead code
#[allow(dead_code)]
pub fn setup() -> Runtime {
    let mut path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    path.push("test");

    let mut data_home = path.clone();
    let mut cache_home = path.clone();
    let mut config_home = path.clone();
    data_home.push("data");
    cache_home.push("cache");
    config_home.push("config");

    std::env::set_var("XDG_DATA_HOME", data_home.as_os_str());
    std::env::set_var("XDG_CACHE_HOME", cache_home.as_os_str());
    std::env::set_var("XDG_CONFIG_HOME", config_home.as_os_str());
    Runtime::new().unwrap()
}
