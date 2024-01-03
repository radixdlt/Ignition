fn main() {
    build_blueprints();
    decompress_state();
}

#[cfg(not(feature = "compile-blueprints-at-build-time"))]
fn build_blueprints() {}

#[cfg(feature = "compile-blueprints-at-build-time")]
fn build_blueprints() {
    use std::env;
    use std::path::PathBuf;

    use cargo_toml::{Manifest, Package};
    use scrypto::prelude::*;

    let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let blueprints_dir = manifest_dir.parent().unwrap();
    println!("cargo:rerun-if-changed=\"{:?}\"", blueprints_dir);

    let mut packages = HashMap::new();
    for entry in walkdir::WalkDir::new(blueprints_dir) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path
            .file_name()
            .map_or(false, |file_name| file_name == "Cargo.toml")
        {
            continue;
        }

        let manifest = Manifest::from_path(path).unwrap();
        if !manifest
            .dependencies
            .into_iter()
            .any(|(name, _)| name == "scrypto")
        {
            continue;
        }

        let Some(Package { name, .. }) = manifest.package else {
            continue;
        };

        let (code, definition) = scrypto_unit::Compile::compile_with_env_vars(
            path.parent().unwrap(),
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
                "LLVM_PROFILE_FILE".to_owned() => "".to_owned()
            },
        );
        packages.insert(name, (code, definition));
    }

    let out_dir =
        PathBuf::from_str(env::var("OUT_DIR").unwrap().as_str()).unwrap();
    let compilation_path = out_dir.join("compiled_packages.bin");

    let encoded_packages = scrypto_encode(&packages).unwrap();
    std::fs::write(compilation_path, encoded_packages).unwrap();
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