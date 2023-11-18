mod csv;

use crate::{config::Config, hq::HQ};

use self::csv::Csv;
use anyhow::Result;
use clap::Subcommand;
use std::ops::Deref;

pub trait Command {
    fn run(&self, hq: &HQ, config: &Config) -> Result<()>;
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Csv(Csv),
}

impl Deref for Commands {
    type Target = dyn Command;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Csv(c) => c,
        }
    }
}
