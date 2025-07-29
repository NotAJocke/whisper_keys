use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/Users/jocke/WhisperKeys/Mammoth75");

    lib::pack::from_mechvibes(&path).unwrap();
}
