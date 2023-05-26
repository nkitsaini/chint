#[warn(clippy::unimplemented)]
mod cli;
mod command_guesser;
mod test_runner;
mod text_diff;

use std::time::Duration;

use anyhow::{bail, Context};
use cli::{Command, SolutionSpec};
use macro_types::Problem;

pub const PROBLEMS: &[Problem] = macros::include_dir!("chint/problems");
type StaticProblem = &'static Problem<'static>;

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
            problem.title.strip_prefix('#').unwrap_or(problem.title),
            problem.tests.len()
        );
    }
}

fn show(problem: StaticProblem) {
    termimad::print_text(&(problem.title.to_string() + "\n" + problem.description));
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
