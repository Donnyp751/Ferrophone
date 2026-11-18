// SPDX-License-Identifier: GPL-2.0-or-later
//! `ferroctl`: command-line client for `ferrophoned`. Commands arrive in M1.

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {}

fn main() -> anyhow::Result<()> {
    let _ = Args::parse();
    eprintln!("ferroctl: no commands yet (M1); see docs/plan.md");
    Ok(())
}
