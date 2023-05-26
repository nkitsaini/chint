mod tester;
use std::{process::exit, time::Duration};

use clap::{Args, Parser, Subcommand};
use macro_types::Problem;

const PROBLEMS: &[Problem] = macros::include_dir!("chint/problems");
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
            for (i, problem) in PROBLEMS.iter().enumerate() {
                println!(
                    "{}: {} (tests: {})",
                    i + 1,
                    problem.title,
                    problem.tests.len()
                );
            }
            return;
        }
        Commands::Problem(x) => x,
    };
    if args.problem_id == 0 {
        eprintln!("Invalid problem number: 0. Problems start with no. 1");
        exit(1);
    }
    let problem = match PROBLEMS.get(args.problem_id - 1) {
        Some(x) => x,
        None => {
            eprintln!(
                "Invalid problem number: {}. There are only {} problems.",
                args.problem_id,
                PROBLEMS.len()
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
            let result =
                tester::test_problem(problem, &command, Duration::from_secs(timeout as u64));
            let success = match result {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Got error while running the tests: {:?}", e);
                    exit(1);
                }
            };
            if success {
                println!("Hooray!!")
            } else {
                println!("Try just once more!!")
            }
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
