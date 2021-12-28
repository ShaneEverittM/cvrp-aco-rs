use std::fs::File;
use std::{fmt::Debug, path::PathBuf};

use anyhow::Result;
use clap::Parser;

use aco::{Problem, Simulator};

mod aco;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    vrp: PathBuf,
}

fn main() -> Result<()> {
    let vrp = File::open(Args::parse().vrp)?;
    let problem = Problem::try_from_vrp(vrp)?;
    Simulator::on(problem).run()
}
