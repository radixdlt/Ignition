fn main() {
    decompress_state();
}

fn decompress_state() {
    use flate2::read::*;

    use std::env;
    use std::io::prelude::*;
    use std::path::*;
    use std::str::FromStr;

    println!("cargo:rerun-if-changed=\"./assets/state\"");
    let compressed = include_bytes!("./assets/state");
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut uncompressed = Vec::new();
    decoder
        .read_to_end(&mut uncompressed)
        .expect("Failed to decompress!");

    let path = PathBuf::from_str(env::var("OUT_DIR").unwrap().as_str())
        .unwrap()
        .join("uncompressed_state.bin");
    std::fs::write(path, uncompressed).unwrap();
}
