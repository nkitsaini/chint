use std::path::Path;

use anyhow::Context;

type FileContent = Vec<u8>;

pub struct Test {
    pub(crate) test_name: String,
    pub(crate) input: FileContent,
    pub(crate) output: FileContent,
}

pub struct Problem {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) tests: Vec<Test>,
}

fn get_file<'a, S: AsRef<Path>>(
    dir: &'a include_dir::Dir,
    file: S,
) -> Option<&'a include_dir::File<'a>> {
    dir.get_file(dir.path().join(file))
}

fn get_dir<'a, S: AsRef<Path>>(
    dir: &'a include_dir::Dir,
    dir_name: S,
) -> Option<&'a include_dir::Dir<'a>> {
    dir.get_dir(dir.path().join(dir_name))
}

impl Problem {
    pub fn from_dir(dir: include_dir::Dir) -> anyhow::Result<Vec<Problem>> {
        let mut rv = vec![];
        for i in 1..u32::MAX {
            if let Some(problem_dir) = get_dir(&dir, i.to_string()) {
                let file = get_file(problem_dir, "description.md").context(format!(
                    "description.md missing from {:?}",
                    problem_dir.path()
                ))?;
                let problem_description = file
                    .contents_utf8()
                    .context(format!("invalid utf-8 in {:?}", file.path()))?;
                let (title, description) = problem_description
                    .split_once('\n')
                    .context(format!("Invalid format in {:?}", file.path()))?;
                let mut problem = Self {
                    title: title.to_string(),
                    description: description.to_string(),
                    tests: vec![],
                };

                for input in problem_dir.files() {
                    let input_path_name = input
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .context("non-str file name")?;

                    if input_path_name.ends_with(".in") {
                        let test_name = input_path_name.strip_suffix(".in").unwrap();
                        let output_file =
                            get_file(problem_dir, format!("{test_name}.out")).context(
                                format!("Missing matching output for input: {input_path_name}"),
                            )?;
                        problem.tests.push(Test {
                            test_name: test_name.to_string(),
                            input: input.contents().to_vec(),
                            output: output_file.contents().to_vec(),
                        })
                    }
                }
                problem.tests.sort_by(|a, b| a.test_name.cmp(&b.test_name));
                rv.push(problem);
            } else {
                break;
            }
        }
        Ok(rv)
    }
}
