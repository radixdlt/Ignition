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

#![allow(dead_code, clippy::enum_variant_names, clippy::wrong_self_convention)]

mod publish;

use clap::Parser;
use publishing_tool::error::*;
use radix_engine_common::prelude::*;
use transaction::prelude::*;

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut out = std::io::stdout();
    let cli = <Cli as clap::Parser>::parse();
    cli.run(&mut out)
}

#[derive(Parser, Debug)]
pub enum Cli {
    Publish(publish::Publish),
}

impl Cli {
    pub fn run<O: std::io::Write>(self, out: &mut O) -> Result<(), Error> {
        match self {
            Self::Publish(cmd) => cmd.run(out),
        }
    }
}
