pub type FileContent = [u8];

pub struct Test<'a> {
    pub test_name: &'a str,
    pub input: &'a FileContent,
    pub output: &'a FileContent,
}

pub struct Problem<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub tests: &'a [Test<'a>],
}
