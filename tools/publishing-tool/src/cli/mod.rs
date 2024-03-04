mod default_configurations;
mod publish;

use crate::Error;
use clap::Parser;

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
