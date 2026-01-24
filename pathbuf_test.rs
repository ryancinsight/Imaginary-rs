use std::path::PathBuf;
fn main() {
    let p = PathBuf::from("test");
    let s = p.to_string();
    println!("{}", s);
}
