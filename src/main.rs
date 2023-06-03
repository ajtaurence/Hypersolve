use clap::{Parser, Subcommand, ValueEnum};
use hypersolve::{piece_cube::{twist::Twist, puzzle::PieceCube}, search::fast_solve};
use itertools::Itertools;

#[derive(Parser)]
#[command(
    author, 
    version,
    about, 
    long_about = None,
    help_template = "{about-section}{author}\n\n{usage-heading} {usage} \n\n{all-args} {tab}"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solves a scramble
    Solve {
        /// Scramble moves
        moves: Vec<Twist>,

        /// Solution mode
        #[arg(short, long, value_enum, default_value_t = SolveMode::Fast)]
        mode: SolveMode,
    },
    /// Simplifies a move sequence
    Simplify {
        /// Scramble moves
        moves: Vec<Twist>,

        /// Simplification mode
        #[arg(short, long, value_enum)]
        mode: SimplifyMode,
    },
    /// Generates true random state scramble
    Scramble {
        /// Random state seed
        #[arg(short, long)]
        seed: Option<u64>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum SolveMode {
    /// Finds short solutions quickly
    Fast,
    /// Finds the optimal solution
    Optimal,
}

#[derive(Clone, Copy, ValueEnum)]
enum SimplifyMode {
    /// Applies trivial simplifications such as combining moves where possible
    Trivial,
    /// Searches for equivalent but shorter move sequences
    NonTrivial,
}

fn main() {
    
    // let args = Cli::parse_from(["hypersolve", "solve", "58,1,1"]);

    // let twist = Twist::from_mc4d_twist_string("58,1,1").unwrap();
    // let twist = Twist::new(twist.face, twist.direction, hypersolve::piece_cube::twist::LayerEnum::Both);
    // let cube = PieceCube::solved().twist(twist);
    // println!("{:?}", cube.pieces.map(|mut piece| {piece.faces.sort(); piece}));

    let args = Cli::parse();
    
    

    match args.command {
        Commands::Solve { moves, mode } => {
            match mode {
                SolveMode::Fast => {
                    let solutions = fast_solve(PieceCube::solved().twists(moves), None);
                    while let Ok((soln, length)) = solutions.recv(){
                        println!("Found solution of length {}: {}", length, soln.into_iter().map(|twist| twist.to_mc4d_string()).join(" "))
                    }
                },
                SolveMode::Optimal => todo!()
            }
        },
        _ => todo!()
    }
}
