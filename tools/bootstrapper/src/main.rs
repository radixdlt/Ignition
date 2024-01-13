//! A tool used to bootstrap and publish project Ignition on various networks
//! and with varying configurations. This is a CLI tool and is made to be run
//! like a script.

#[macro_use]
mod macros;
mod cli;
mod constants;
mod error;
mod gateway;
mod subcommands;
mod utils;

use clap::Parser;
use error::*;

fn main() -> Result<(), Error> {
    let mut out = std::io::stdout();
    let cli = cli::Cli::parse();
    cli.run(&mut out)
}
