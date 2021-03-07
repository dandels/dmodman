pub fn vec_with_format_string(format_string: &str, params: Vec<&str>) -> String {
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

pub fn bytes_as_unit(bytes: u64, unit: usize) -> String {
    let mut bytes: f64 = bytes as f64;
    let mut i = 0;
    while i < unit {
        bytes /= 1024.0;
        i += 1;
    }
    format!("{:.*}", 1, bytes)
}

pub fn human_readable(bytes: u64) -> (String, usize) {
    let mut bytes: f64 = bytes as f64;
    let units = vec!["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let mut i = 0;
    while (bytes * 10.0).round() / 10.0 >= 1024.0 && i < units.len() - 1 {
        bytes /= 1024.0;
        i += 1;
    }
    if i == 0 {
        return (format!("{} {}", bytes as u64, units[i]), 0);
    }
    (format!("{:.*} {}", 1, bytes, units[i]), i)
}

#[cfg(test)]
mod tests {
    use crate::util::format;

    #[test]
    fn endpoint_format() {
        let arg = "games/{}/mods/{}/files.json";
        let params = vec!["morrowind", "46599"];

        assert_eq!(
            "games/morrowind/mods/46599/files.json",
            format::vec_with_format_string(&arg, params)
        );
    }

    #[test]
    fn human_readable() {
        assert_eq!("272 B", format::human_readable(272).0);
        assert_eq!("83.4 KiB", format::human_readable(85417).0);
        assert_eq!("204.1 MiB", format::human_readable(214022328).0);
        assert_eq!("936.7 MiB", format::human_readable(982232812).0);
        assert_eq!("19.9 GiB", format::human_readable(21402232812).0);
    }
}
