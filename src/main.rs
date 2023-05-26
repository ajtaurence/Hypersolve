use clap::Parser;
use hypersolve::{
    phases::{Phase1, Phase2, Phase3},
    prune::{explore, gen_pruning_table, ArrayPruningTable},
};

/// Input an MC4D log file
#[derive(Parser)]
struct Cli {
    /// The path to the log file
    path: std::path::PathBuf,
}

fn main() {
    // let args = Cli::parse();

    let table = explore::<Phase2>();
    println!("{:?}", table[0])
}
