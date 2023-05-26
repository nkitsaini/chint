#[warn(clippy::unimplemented)]
mod cli;
mod command_guesser;
mod test_runner;

use std::time::Duration;

use anyhow::{bail, Context};
use clap::{Args, Parser, Subcommand};
use cli::{Command, SolutionSpec};
use macro_types::Problem;

pub const PROBLEMS: &[Problem] = macros::include_dir!("chint/problems");
type StaticProblem = &'static Problem<'static>;

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

fn main() -> anyhow::Result<()> {
    let command = cli::get_args();

    match command {
        Command::List => list(),
        Command::Show { problem } => show(problem),
        Command::Test {
            problem,
            spec,
            timeout,
        } => {
            test(problem, spec, timeout)?;
        }
    };

    Ok(())
}

fn list() {
    for (i, problem) in PROBLEMS.iter().enumerate() {
        println!(
            "{}: {} (tests: {})",
            i + 1,
            problem.title,
            problem.tests.len()
        );
    }
}

fn show(problem: StaticProblem) {
    println!("Title: {}\n", problem.title);
    println!("Description:");
    println!("{}", problem.description);
}

fn test(problem: StaticProblem, spec: SolutionSpec, timeout: Duration) -> anyhow::Result<()> {
    let command = match spec {
        cli::SolutionSpec::File(f) => command_guesser::guess_command(&f)
            .context("Unsupported file format, please provide full command using -c arg")?,
        cli::SolutionSpec::Command(c) => c,
    };
    let result = test_runner::test_problem(problem, &command, timeout);
    let success = match result {
        Ok(x) => x,
        Err(e) => {
            bail!("Got error while running the tests: {:?}", e);
        }
    };
    if success {
        println!("Hooray!!");
    } else {
        println!("Try just once more!!");
    }
    Ok(())
}
