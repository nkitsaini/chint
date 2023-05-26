use std::io;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use crate::PROBLEMS;
use clap::{arg, command, Args, Parser, Subcommand};
use clap::{value_parser, CommandFactory};
use macro_types::Problem;

type ProblemId = u64;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct _Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// List all the available problems
    /// Example:
    /// 	chint list
    #[clap(verbatim_doc_comment)]
    List,

    Test(TestCommand),

    /// Show description of a problem
    /// Examples:
    /// 	chint show 1
    /// 	chint show 10
    #[clap(verbatim_doc_comment)]
    Show(ShowCommand),

    /// Generate shell completions
    /// Examples:
    /// 	chint completion bash
    /// 	chint completion fish
    Completion {
        shell: clap_complete::Shell,
    },
}

#[derive(Args)]
struct ShowCommand {
    #[arg(value_parser = value_parser!(u64).range(1..PROBLEMS.len() as u64+1))]
    problem_id: u64,
}

const TEST_HELP: &'static str = r#"
Usage:
Test your solution for a problem
To test solution for problem no. 1 you can run:
	chint test 1 <python file python>
For files other then python you can use
	chint test 1 -c "<command to run your program>"
Examples:
	chint test 1 -c "node sol.js"
	chint test 1 main.py
	chint test 1 my_solutions/main.py
	chint test 1 -c "echo Hello World"
"#;

#[derive(Args)]
#[clap()]
struct TestCommand {
    #[arg(value_parser = value_parser!(u64).range(1..))]
    problem_id: u64,

    /// Seconds to wait for solution to complete
    #[arg(short, long, default_value_t = 60)]
    timeout: u64,

    #[command(flatten)]
    sol: _SolutionSpec,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct _SolutionSpec {
    file: Option<PathBuf>,
    #[arg(short, long)]
    command: Option<String>,
}

pub enum Command {
    List,
    Show {
        problem: &'static Problem<'static>,
    },
    Test {
        problem: &'static Problem<'static>,
        spec: SolutionSpec,
        timeout: Duration,
    },
}
fn get_problem(id: ProblemId) -> &'static Problem<'static> {
    &PROBLEMS[id as usize - 1]
}

impl From<_Cli> for Command {
    fn from(value: _Cli) -> Self {
        match value.command {
            CliCommand::List => Self::List,
            CliCommand::Show(show) => Self::Show {
                problem: get_problem(show.problem_id),
            },
            CliCommand::Test(test) => {
                if let Some(command) = test.sol.command {
                    Self::Test {
                        problem: get_problem(test.problem_id),
                        spec: SolutionSpec::Command(command),
                        timeout: Duration::from_secs(test.timeout),
                    }
                } else if let Some(file) = test.sol.file {
                    Self::Test {
                        problem: get_problem(test.problem_id),
                        spec: SolutionSpec::File(file),
                        timeout: Duration::from_secs(test.timeout),
                    }
                } else {
                    unreachable!()
                }
            }
            CliCommand::Completion { shell } => {
                let mut cli = _Cli::command();
                let name = cli.get_name().to_string();
                clap_complete::generate(shell, &mut cli, name, &mut io::stdout());
                exit(0);
            }
        }
    }
}

pub enum SolutionSpec {
    Command(String),
    File(PathBuf),
}

pub fn get_args() -> Command {
    return _Cli::parse().into();
}

#[test]
fn verify_cli() {
    _Cli::command().debug_assert();
}
