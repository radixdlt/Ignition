use crate::error::*;
use crate::subcommands::*;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Cli {
    /// Publishes project Ignition to stokenet with the test configuration.
    PublishTestToStokenet(publish_test_to_stokenet::PublishTestToStokenet),
}

impl Cli {
    pub fn run<O: std::io::Write>(self, out: &mut O) -> Result<(), Error> {
        match self {
            Self::PublishTestToStokenet(cmd) => cmd.run(out),
        }
    }
}
