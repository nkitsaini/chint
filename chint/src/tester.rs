use anyhow::Context;
use macro_types::{Problem, Test};
use shlex::split;
use std::io::{stderr, BufReader, Read};
use std::time::Duration;
use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
};
use wait_timeout::ChildExt;

enum ResultStatus {
    Success,
    IncorrectExitCode { exit_code: i32 },
    IncorrectOutput,
    Timeout,
}
struct Result {
    time_taken: Duration,
    stdout: String,
    stderr: String,
    status: ResultStatus,
}

fn run_test(test: &Test, command: &str, timeout: Duration) -> anyhow::Result<Result> {
    let a = split(command).context("Invalid Command")?;

    let mut rust_command = Command::new(a.get(0).context("Empty Command")?);
    rust_command.args(&a[1..]);
    rust_command.stdin(Stdio::piped());
    rust_command.stdout(Stdio::piped());
    rust_command.stderr(Stdio::piped());
    let mut child = rust_command.spawn()?;
    child.stdin.take().unwrap().write_all(test.input)?;
    let start = std::time::Instant::now();
    let r = child.wait_timeout(timeout)?;
    let end = std::time::Instant::now();
    let duration = end - start;

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    // stdout
    let mut reader = BufReader::new(&mut stdout);
    let mut output = vec![];
    reader.read_to_end(&mut output)?;

    // stderr
    let mut reader = BufReader::new(&mut stderr);
    let mut error = vec![];
    reader.read_to_end(&mut error)?;

    let error = String::from_utf8_lossy(&error).to_string();
    let output = String::from_utf8_lossy(&output).to_string();

    let exit_status = match r {
        Some(e) => e,
        None => {
            return Ok(Result {
                time_taken: duration,
                stdout: output,
                stderr: error,
                status: ResultStatus::Timeout,
            })
        }
    };

    if !exit_status.success() {
        return Ok(Result {
            time_taken: duration,
            stderr: error,
            stdout: output,
            status: ResultStatus::IncorrectExitCode {
                exit_code: exit_status.code().context("Exited without exit code")?,
            },
        });
    }

    // TODO: stricter match
    if output.trim_end() == String::from_utf8_lossy(test.output).trim_end() {
        Ok(Result {
            time_taken: duration,
            stderr: error,
            stdout: output,
            status: ResultStatus::Success,
        })
    } else {
        Ok(Result {
            time_taken: duration,
            stderr: error,
            stdout: output,
            status: ResultStatus::IncorrectOutput,
        })
    }
}

/// returns if tests were successful or not
pub fn test_problem(problem: &Problem, command: &str, timeout: Duration) -> anyhow::Result<bool> {
    for (i, test) in problem.tests.iter().enumerate() {
        println!("=== [{}/{}]", i, problem.tests.len());
        let result = run_test(test, command, timeout)?;
        println!("=== Time: {:.4}s", result.time_taken.as_secs_f64());
        match result.status {
            ResultStatus::Success => {
                println!("Success")
            }
            ResultStatus::Timeout => {
                eprintln!("Test Timed out");
                return Ok(false);
            }
            ResultStatus::IncorrectExitCode { exit_code } => {
                eprintln!("Incorrect Exit Code: {}", exit_code);
                if result.stdout.len() != 0 {
                    eprintln!("---------------- Stdout: ");
                    println!("{}", result.stdout);
                }
                if result.stderr.len() != 0 {
                    eprintln!("---------------- Stderr: ");
                    println!("{}", result.stderr);
                }
                return Ok(false);
            }
            ResultStatus::IncorrectOutput => {
                eprintln!("Incorrect Output: ");
                eprintln!("---------------- Expected: ");
                println!("{}", String::from_utf8_lossy(test.output));
                eprintln!("---------------- Got: ");
                println!("{}", result.stdout);
                if result.stderr.len() != 0 {
                    eprintln!("---------------- Stderr: ");
                    println!("{}", result.stderr);
                }
                return Ok(false);
            }
        }
    }
    Ok(true)
}
