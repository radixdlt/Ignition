#![allow(dead_code, clippy::enum_variant_names)]

#[macro_use]
mod macros;
mod cli;
mod error;
mod mainnet_testing;
mod stokenet_production;
mod transaction_service;
mod types;

fn main() -> Result<(), error::Error> {
    let mut out = std::io::stdout();
    let cli = <cli::Cli as clap::Parser>::parse();
    cli.run(&mut out)
}
