use crate::error::*;
use crate::stokenet_production;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Cli {
    StokenetProduction(stokenet_production::StokenetProduction),
}

impl Cli {
    pub fn run<O: std::io::Write>(self, out: &mut O) -> Result<(), Error> {
        match self {
            Self::StokenetProduction(cmd) => cmd.run(out),
        }
    }
}
