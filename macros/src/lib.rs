//! Modified from https://github.com/Michael-F-Bryan/include_dir/blob/master/macros/src/lib.rs
//! Implementation details of the `include_dir`.
//!
//! You probably don't want to use this crate directly.
#![cfg_attr(feature = "nightly", feature(track_path, proc_macro_tracked_env))]

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

/// Embed the contents of a directory in your crate.
#[proc_macro]
pub fn include_dir(input: TokenStream) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();

    let path = match tokens.as_slice() {
        [TokenTree::Literal(lit)] => unwrap_string_literal(lit),
        _ => panic!("This macro only accepts a single, non-empty string argument"),
    };

    let path = resolve_path(&path, get_env).unwrap();

    expand_dir(&path, &path).into()
}

fn unwrap_string_literal(lit: &proc_macro::Literal) -> String {
    let mut repr = lit.to_string();
    if !repr.starts_with('"') || !repr.ends_with('"') {
        panic!("This macro only accepts a single, non-empty string argument")
    }

    repr.remove(0);
    repr.pop();

    repr
}

fn expand_dir(root: &Path, path: &Path) -> proc_macro2::TokenStream {
    let children = read_dir(path).unwrap_or_else(|e| {
        panic!(
            "Unable to read the entries in \"{}\": {}",
            path.display(),
            e
        )
    });
    let mut number_to_dir = HashMap::new();
    for child in children {
        let name = child.file_name().unwrap().to_str().unwrap();
        if let Ok(x) = name.parse::<u64>() {
            number_to_dir.insert(x, child);
        }
    }
    let mut problems = vec![];

    for i in 1..u64::MAX {
        let dir = match number_to_dir.get(&i) {
            Some(x) => x,
            None => break,
        };
        let mut childs = read_dir(dir).unwrap_or_else(|e| {
            panic!(
                "Unable to read the entries in \"{}\": {}",
                path.display(),
                e
            )
        });
        let mut problem_description = None;
        let mut tests = vec![];
        let mut current_test_name: Option<String> = None;
        let mut current_test_input: Option<Vec<u8>> = None;
        childs.sort_by(|a, b| a.file_name().unwrap().cmp(b.file_name().unwrap()));
        for child in childs {
            let name = child.file_name().unwrap().to_str().unwrap();

            match name {
                "description.md" => {
                    let content = String::from_utf8(read_file(&child)).unwrap();
                    problem_description = Some(content);
                }
                x if x.ends_with(".out") => {
                    let problem_name = x.strip_suffix(".out").unwrap();
                    assert_eq!(
                        problem_name,
                        &current_test_name
                            .clone()
                            .expect("Out file without input file")
                    );
                    let name = current_test_name.expect("Outfile without input file");
                    let input = current_test_input.unwrap();
                    let output = read_file(&child);
                    tests.push(quote! {macro_types::Test {
                        test_name: #name,
                        input: &[#(#input), *],
                        output: &[#(#output), *],
                    }});
                    current_test_name = None;
                    current_test_input = None;
                }
                x if x.ends_with(".in") => {
                    let problem_name = x.strip_suffix(".in").unwrap();
                    assert!(
                        current_test_input.is_none() && current_test_name.is_none(),
                        "Possibly missing output file for {current_test_name:?}"
                    );
                    current_test_name = Some(problem_name.to_string());
                    current_test_input = Some(read_file(&child));
                }
                _ => panic!("Unexpected file {name} in {child:?}"),
            }
        }
        let problem_desc = problem_description.expect("description.md missing");
        let (title, description) = problem_desc
            .split_once('\n')
            .expect("Invalid format in description.md");

        problems.push(quote! {
            macro_types::Problem {
                title: #title,
                description: #description,
                tests: &[#(#tests), *],
            }
        });

        let _description_path = dir.join("description.md");
        let _description_path = dir.join("description.md");
    }

    let _path = normalize_path(root, path);

    quote! {
        &[ #(#problems),*]
    }
}

/// Make sure that paths use the same separator regardless of whether the host
/// machine is Windows or Linux.
fn normalize_path(root: &Path, path: &Path) -> String {
    let stripped = path
        .strip_prefix(root)
        .expect("Should only ever be called using paths inside the root path");
    let as_string = stripped.to_string_lossy();

    as_string.replace('\\', "/")
}

fn read_dir(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !dir.is_dir() {
        panic!("\"{}\" is not a directory", dir.display());
    }

    track_path(dir);

    let mut paths = Vec::new();

    for entry in dir.read_dir()? {
        let entry = entry?;
        paths.push(entry.path());
    }

    paths.sort();

    Ok(paths)
}

fn read_file(path: &Path) -> Vec<u8> {
    track_path(path);
    std::fs::read(path).unwrap_or_else(|e| panic!("Unable to read \"{}\": {}", path.display(), e))
}

fn resolve_path(
    raw: &str,
    get_env: impl Fn(&str) -> Option<String>,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut unprocessed = raw;
    let mut resolved = String::new();

    while let Some(dollar_sign) = unprocessed.find('$') {
        let (head, tail) = unprocessed.split_at(dollar_sign);
        resolved.push_str(head);

        match parse_identifier(&tail[1..]) {
            Some((variable, rest)) => {
                let value = get_env(variable).ok_or_else(|| MissingVariable {
                    variable: variable.to_string(),
                })?;
                resolved.push_str(&value);
                unprocessed = rest;
            }
            None => {
                return Err(UnableToParseVariable { rest: tail.into() }.into());
            }
        }
    }
    resolved.push_str(unprocessed);

    Ok(PathBuf::from(resolved))
}

#[derive(Debug, PartialEq)]
struct MissingVariable {
    variable: String,
}

impl Error for MissingVariable {}

impl Display for MissingVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to resolve ${}", self.variable)
    }
}

#[derive(Debug, PartialEq)]
struct UnableToParseVariable {
    rest: String,
}

impl Error for UnableToParseVariable {}

impl Display for UnableToParseVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to parse a variable from \"{}\"", self.rest)
    }
}

fn parse_identifier(text: &str) -> Option<(&str, &str)> {
    let mut calls = 0;

    let (head, tail) = take_while(text, |c| {
        calls += 1;

        match c {
            '_' => true,
            letter if letter.is_ascii_alphabetic() => true,
            digit if digit.is_ascii_digit() && calls > 1 => true,
            _ => false,
        }
    });

    if head.is_empty() {
        None
    } else {
        Some((head, tail))
    }
}

fn take_while(s: &str, mut predicate: impl FnMut(char) -> bool) -> (&str, &str) {
    let mut index = 0;

    for c in s.chars() {
        if predicate(c) {
            index += c.len_utf8();
        } else {
            break;
        }
    }

    s.split_at(index)
}

#[cfg(feature = "nightly")]
fn get_env(variable: &str) -> Option<String> {
    proc_macro::tracked_env::var(variable).ok()
}

#[cfg(not(feature = "nightly"))]
fn get_env(variable: &str) -> Option<String> {
    std::env::var(variable).ok()
}

fn track_path(_path: &Path) {
    #[cfg(feature = "nightly")]
    proc_macro::tracked_path::path(_path.to_string_lossy());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_with_no_environment_variables() {
        let path = "./file.txt";

        let resolved = resolve_path(path, |_| unreachable!()).unwrap();

        assert_eq!(resolved.to_str().unwrap(), path);
    }

    #[test]
    fn simple_environment_variable() {
        let path = "./$VAR";

        let resolved = resolve_path(path, |name| {
            assert_eq!(name, "VAR");
            Some("file.txt".to_string())
        })
        .unwrap();

        assert_eq!(resolved.to_str().unwrap(), "./file.txt");
    }

    #[test]
    fn dont_resolve_recursively() {
        let path = "./$TOP_LEVEL.txt";

        let resolved = resolve_path(path, |name| match name {
            "TOP_LEVEL" => Some("$NESTED".to_string()),
            "$NESTED" => unreachable!("Shouldn't resolve recursively"),
            _ => unreachable!(),
        })
        .unwrap();

        assert_eq!(resolved.to_str().unwrap(), "./$NESTED.txt");
    }

    #[test]
    fn parse_valid_identifiers() {
        let inputs = vec![
            ("a", "a"),
            ("a_", "a_"),
            ("_asf", "_asf"),
            ("a1", "a1"),
            ("a1_#sd", "a1_"),
        ];

        for (src, expected) in inputs {
            let (got, rest) = parse_identifier(src).unwrap();
            assert_eq!(got.len() + rest.len(), src.len());
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn unknown_environment_variable() {
        let path = "$UNKNOWN";

        let err = resolve_path(path, |_| None).unwrap_err();

        let missing_variable = err.downcast::<MissingVariable>().unwrap();
        assert_eq!(
            *missing_variable,
            MissingVariable {
                variable: String::from("UNKNOWN"),
            }
        );
    }

    #[test]
    fn invalid_variables() {
        let inputs = &["$1", "$"];

        for input in inputs {
            let err = resolve_path(input, |_| unreachable!()).unwrap_err();

            let err = err.downcast::<UnableToParseVariable>().unwrap();
            assert_eq!(
                *err,
                UnableToParseVariable {
                    rest: input.to_string(),
                }
            );
        }
    }
}
