// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

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
    use radix_common::prelude::*;

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

    println!("cargo:rerun-if-changed={}", packages_path.display());

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

    // Build 1: Building each of the packages with the package definition.
    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--target-dir",
            builds_target_path.as_path().display().to_string().as_str(),
            "--features",
            "scrypto/log-info",
        ])
        .args(package_names.iter().flat_map(|package_name| {
            ["--package".to_owned(), package_name.to_owned()]
        }))
        .status()?;
    if !status.success() {
        return Err(Error::CompilationOfPackageFailed);
    }

    // Read the package definition of the various packages - assume code to be
    // an empty byte array for now.
    let mut packages = package_names
        .iter()
        .map(|package_name| {
            let wasm_path = builds_target_path
                .join("wasm32-unknown-unknown")
                .join("release")
                .join(format!("{}.wasm", package_name.replace('-', "_")));

            let package_definition =
                radix_engine::utils::extract_definition(&read(wasm_path)?)?;

            Ok::<_, Error>((
                package_name.clone(),
                (Vec::<u8>::new(), package_definition),
            ))
        })
        .collect::<Result<IndexMap<_, _>, _>>()?;

    // Build 2: Build without the package definition.
    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--target-dir",
            builds_target_path.as_path().display().to_string().as_str(),
            "--features",
            "scrypto/no-schema",
            "--features",
            "scrypto/log-info",
        ])
        .args(package_names.iter().flat_map(|package_name| {
            ["--package".to_owned(), package_name.to_owned()]
        }))
        .status()?;
    if !status.success() {
        return Err(Error::CompilationOfPackageFailed);
    }

    for package_name in package_names.iter() {
        let wasm_path = builds_target_path
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", package_name.replace('-', "_")));

        // Optimize the WASM using wasm-opt for size
        wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
            .add_pass(wasm_opt::Pass::StripDebug)
            .add_pass(wasm_opt::Pass::StripDwarf)
            .add_pass(wasm_opt::Pass::StripProducers)
            .run(&wasm_path, &wasm_path)?;

        let wasm = read(wasm_path)?;

        packages.get_mut(package_name.as_str()).unwrap().0 = wasm;
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
    CompilationOfPackageFailed,
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
