fn main() -> Result<(), Error> {
    build_blueprints()?;
    Ok(())
}

#[cfg(not(feature = "build-time-blueprints"))]
fn build_blueprints() -> Result<(), Error> {
    Ok(())
}

#[cfg(feature = "build-time-blueprints")]
fn build_blueprints() -> Result<(), Error> {
    use std::env::*;
    use std::fs::*;
    use std::path::*;
    use std::process::*;
    use std::*;

    use cargo_toml::Manifest;
    use radix_engine_interface::prelude::*;

    // All of the blueprints are in the `packages` subdirectory of the project.
    // So, we get the path to it so that we can start finding the blueprints
    // here.
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or(Error::FailedToFindAncestor {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            ancestor: 2,
        })?
        .to_owned();

    let packages_path = root_path.join("packages");
    let target_path = root_path.join("target");
    let builds_target_path = target_path.join("package-loader-target");
    if !builds_target_path.exists() {
        create_dir(&builds_target_path)?;
    }

    // Getting the name of all of the blueprints found in the packages directory
    let package_names = read_dir(packages_path)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if entry.file_type().is_ok_and(|ty| ty.is_dir()) {
                Some(entry.path())
            } else {
                None
            }
        })
        .map(|path| {
            Manifest::from_path(path.join("Cargo.toml"))
                .map(|manifest| manifest.package.map(|package| package.name))
        })
        .filter_map(Result::ok)
        .flatten()
        .collect::<Vec<_>>();

    // Building each of the packages that have been discovered.
    let mut packages = HashMap::new();
    for package_name in package_names {
        // Build the package
        let status = Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--target-dir",
                builds_target_path.as_path().display().to_string().as_str(),
                "--package",
                package_name.as_str(),
            ])
            .status()?;
        if !status.success() {
            return Err(Error::CompilationOfPackageFailed(package_name));
        }

        // Construct the path to the WASM file.
        let wasm_path = builds_target_path
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", package_name.replace('-', "_")));

        // Extract the package definition
        let package_definition =
            radix_engine::utils::extract_definition(&read(&wasm_path)?)?;

        // Build a new WASM build without any of the schema information
        let status = Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--target-dir",
                builds_target_path.as_path().display().to_string().as_str(),
                "--package",
                package_name.as_str(),
                "--features",
                "scrypto/no-schema",
            ])
            .status()?;
        if !status.success() {
            return Err(Error::CompilationOfPackageFailed(package_name));
        }

        // Optimize the WASM using wasm-opt for size
        wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
            .add_pass(wasm_opt::Pass::StripDebug)
            .add_pass(wasm_opt::Pass::StripDwarf)
            .add_pass(wasm_opt::Pass::StripProducers)
            .run(&wasm_path, &wasm_path)?;

        // Read the final wasm.
        let wasm = read(wasm_path)?;

        packages.insert(package_name, (wasm, package_definition));
    }

    let out_dir =
        PathBuf::from(var("OUT_DIR").expect("out dir must be defined!"));
    let compilation_path = out_dir.join("compiled_packages.bin");

    let encoded_packages = scrypto_encode(&packages).unwrap();
    write(compilation_path, encoded_packages).unwrap();

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    FailedToFindAncestor {
        path: std::path::PathBuf,
        ancestor: usize,
    },
    IoError(std::io::Error),
    #[cfg(feature = "build-time-blueprints")]
    ManifestError(cargo_toml::Error),
    CompilationOfPackageFailed(String),
    #[cfg(feature = "build-time-blueprints")]
    ExtractSchemaError(radix_engine::utils::ExtractSchemaError),
    #[cfg(feature = "build-time-blueprints")]
    OptimizationError(wasm_opt::OptimizationError),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[cfg(feature = "build-time-blueprints")]
impl From<cargo_toml::Error> for Error {
    fn from(value: cargo_toml::Error) -> Self {
        Self::ManifestError(value)
    }
}

#[cfg(feature = "build-time-blueprints")]
impl From<radix_engine::utils::ExtractSchemaError> for Error {
    fn from(value: radix_engine::utils::ExtractSchemaError) -> Self {
        Self::ExtractSchemaError(value)
    }
}

#[cfg(feature = "build-time-blueprints")]
impl From<wasm_opt::OptimizationError> for Error {
    fn from(value: wasm_opt::OptimizationError) -> Self {
        Self::OptimizationError(value)
    }
}
