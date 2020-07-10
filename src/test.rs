use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;

#[allow(dead_code)] // Used only by tests, so the compiler warns about dead code
pub fn setup() -> Runtime {
    static CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from_str(CRATE_DIR).unwrap();
    path.push("test");
    // TODO set variables for Windows and figure out how MacOS does things
    std::env::set_var("XDG_DATA_HOME", path.as_os_str());
    std::env::set_var("XDG_CONFIG_HOME", path.as_os_str());
    Runtime::new().unwrap()
}
