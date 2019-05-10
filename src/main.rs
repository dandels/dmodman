extern crate chrono;
extern crate reqwest;

mod cmdline;
mod config;
mod download;
mod file;
mod log;
mod mod_info;

fn main() {
    let o: Option<(String, u32)> = cmdline::check_args();
    if !o.is_some() {
        return;
    }
    let t = o.unwrap();
    let game: String = t.0;
    let mod_id: u32 = t.1;

    let mi = download::get_mod_info(&game, &mod_id).expect("");
    println!("{}", mi.name);
}
