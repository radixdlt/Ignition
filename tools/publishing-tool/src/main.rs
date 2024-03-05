#![allow(dead_code, clippy::enum_variant_names, clippy::wrong_self_convention)]

mod cli;
mod database_overlay;
mod error;
mod network_connection_provider;
mod publishing;
mod utils;
#[macro_use]
mod macros;

use error::*;
use network_connection_provider::*;
use publishing::*;
use radix_engine_common::prelude::*;
use transaction::prelude::*;

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut out = std::io::stdout();
    let cli = <cli::Cli as clap::Parser>::parse();
    cli.run(&mut out)
}
