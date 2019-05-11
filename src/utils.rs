use pkg_version;

const MAJOR: u32 = pkg_version::pkg_version_major!();
const MINOR: u32 = pkg_version::pkg_version_minor!();
const PATCH: u32 = pkg_version::pkg_version_patch!();

pub fn get_version() -> String {
    String::from(MAJOR.to_string()) + "." + &MINOR.to_string() + "." + &PATCH.to_string()
}
