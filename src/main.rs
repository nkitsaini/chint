mod problem;

const PROBLEM_DIR: include_dir::Dir<'static> = include_dir::include_dir!("problems");

use std::process::exit;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The problem number
    problem: usize,

    /// The problem number
    #[command(subcommand)]
    command: Commands,

    /// Timeout in seconds
    #[arg(short, long, default_value_t = 1)]
    timeout: usize,
}

#[derive(Subcommand)]
enum Commands {
    Show,
    Test { command: String },
}

fn main() {
    let problems = problem::Problem::from_dir(PROBLEM_DIR).unwrap();
    let args = Cli::parse();
    let problem = match problems.get(args.problem) {
        Some(x) => x,
        None => {
            eprintln!(
                "Invalid problem number: {}. There are only {} problems.",
                args.problem,
                problems.len()
            );
            exit(1);
        }
    };

    match args.command {
        Commands::Show => {
            println!("{}", problem.title);
            println!("{}", problem.description);
        }
        Commands::Test { command } => {}
    }

    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_test_valid_problem_structure() {
        assert!(problem::Problem::from_dir(PROBLEM_DIR).unwrap().len() > 0);
    }
}
