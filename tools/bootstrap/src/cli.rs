use crate::error::*;
use crate::mainnet_testing;
use crate::stokenet_production;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Cli {
    StokenetProduction(stokenet_production::StokenetProduction),
    MainnetTesting(mainnet_testing::MainnetTesting),
}

impl Cli {
    pub fn run<O: std::io::Write>(self, out: &mut O) -> Result<(), Error> {
        match self {
            Self::StokenetProduction(cmd) => cmd.run(out),
            Self::MainnetTesting(cmd) => cmd.run(out),
        }
    }
}
