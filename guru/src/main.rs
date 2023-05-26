mod problem;

use std::process::exit;

use clap::{Args, Parser, Subcommand};
use macro_types::Problem;

const problems: &'static [Problem] = macros::include_dir!("guru/problems");
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The problem number
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
struct TestCommand {
    command: String,
    #[arg(short, long, default_value_t = 1)]
    timeout: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// List all problems
    List,
    /// The problem number
    Problem(ProblemCommand),
}

#[derive(Args)]
struct ProblemCommand {
    problem_id: usize,

    #[command(subcommand)]
    command: ProblemSubCommand,
}

#[derive(Subcommand)]
enum ProblemSubCommand {
    Show,
    Test(TestCommand),
}

fn main() {
    let args = Cli::parse();
    let args = match args.command {
        Commands::List => {
            for (i, problem) in problems.iter().enumerate() {
                println!("{}: {}", i + 1, problem.title);
            }
            return;
        }
        Commands::Problem(x) => x,
    };
    if args.problem_id == 0 {
        eprintln!("Invalid problem number: 0. Problems start with no. 1");
        exit(1);
    }
    let problem = match problems.get(args.problem_id - 1) {
        Some(x) => x,
        None => {
            eprintln!(
                "Invalid problem number: {}. There are only {} problems.",
                args.problem_id,
                problems.len()
            );
            exit(1);
        }
    };
    match args.command {
        ProblemSubCommand::Show => {
            println!("Title: {}\n", problem.title);
            println!("Description:");
            println!("{}", problem.description);
        }
        ProblemSubCommand::Test(TestCommand { command, timeout }) => {
            println!("Running test with {}, timeout: {}", command, timeout);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn build_test_valid_problem_structure() {
//         assert!(problem::Problem::from_dir(PROBLEM_DIR).unwrap().len() > 0);
//     }
// }
