use rkyv::{Archive, Deserialize, Serialize, with::AsString};
use std::path::PathBuf;

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
struct TestStruct {
    #[with(AsString)]
    path: PathBuf,
}

fn main() {}
