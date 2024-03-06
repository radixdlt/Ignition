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
