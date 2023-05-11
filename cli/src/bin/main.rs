use anyhow::Result;
use clap::Parser;
use light_anchor_cli::Opts;

fn main() -> Result<()> {
    light_anchor_cli::entry(Opts::parse())
}
