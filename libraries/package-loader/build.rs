fn main() {
    build_blueprints();
}

#[cfg(not(feature = "build-time-blueprints"))]
fn build_blueprints() {}

#[cfg(feature = "build-time-blueprints")]
fn build_blueprints() {
    use std::env;
    use std::path::PathBuf;

    use cargo_toml::{Manifest, Package};
    use radix_engine_interface::prelude::*;

    let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let blueprints_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packages");
    println!("cargo:rerun-if-changed=\"{:?}\"", blueprints_dir);

    let mut scrypto_packages_manifest_paths = vec![];
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

        scrypto_packages_manifest_paths
            .push((name, path.parent().unwrap().to_owned()));
    }

    let mut packages = HashMap::new();
    for (name, manifest_file_path) in scrypto_packages_manifest_paths {
        let (code, definition) =
            scrypto_unit::Compile::compile(manifest_file_path);
        packages.insert(name, (code, definition));
    }

    let out_dir =
        PathBuf::from_str(env::var("OUT_DIR").unwrap().as_str()).unwrap();
    let compilation_path = out_dir.join("compiled_packages.bin");

    let encoded_packages = scrypto_encode(&packages).unwrap();
    std::fs::write(compilation_path, encoded_packages).unwrap();
}