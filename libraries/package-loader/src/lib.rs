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

#[cfg(feature = "build-time-blueprints")]
#[allow(unused, clippy::module_inception)]
mod package_loader {
    use radix_engine_common::prelude::*;
    use radix_engine_queries::typed_substate_layout::*;
    use std::sync::*;

    const PACKAGES_BINARY: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/compiled_packages.bin"));

    static PACKAGES: OnceLock<HashMap<String, (Vec<u8>, PackageDefinition)>> =
        OnceLock::new();

    pub struct PackageLoader;
    impl PackageLoader {
        pub fn get(name: &str) -> (Vec<u8>, PackageDefinition) {
            let packages = PACKAGES
                .get_or_init(|| scrypto_decode(PACKAGES_BINARY).unwrap());
            if let Some(rtn) = packages.get(name) {
                rtn.clone()
            } else {
                panic!("Package \"{}\" not found. Are you sure that this package is: a) in the blueprints folder, b) that this is the same as the package name in the Cargo.toml file?", name)
            }
        }
    }
}

#[cfg(not(feature = "build-time-blueprints"))]
#[allow(unused, clippy::module_inception)]
mod package_loader {
    use radix_engine_common::prelude::*;
    use radix_engine_queries::typed_substate_layout::*;
    use std::path::PathBuf;

    pub struct PackageLoader;
    impl PackageLoader {
        pub fn get(name: &str) -> (Vec<u8>, PackageDefinition) {
            let package_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("packages")
                .join(name);
            scrypto_unit::Compile::compile(package_dir)
        }
    }
}

pub use package_loader::PackageLoader;
