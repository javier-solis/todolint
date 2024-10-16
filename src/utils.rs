use serde_json;

pub fn print_json<T: serde::Serialize>(item: &T) {
    let json = serde_json::to_string_pretty(item).unwrap();
    println!("{}\n", json);
}
