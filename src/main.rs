use clap::Parser;

/// Input an MC4D log file
#[derive(Parser)]
struct Cli {
    /// The path to the log file
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
}
