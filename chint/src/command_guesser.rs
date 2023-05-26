use shlex;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

pub fn guess_command(file_path: &Path) -> Option<String> {
    if file_path.extension()? == "py" {
        return Some(shlex::join(["python3", file_path.to_str()?]));
    }
    None
}

#[test]
fn guess_command_test() {
    assert!(guess_command(&PathBuf::from_str("abc.xyz").unwrap()).is_none());
    assert_eq!(
        guess_command(&PathBuf::from_str("abc.py").unwrap()),
        Some("python3 abc.py".into())
    );
    assert_eq!(
        guess_command(&PathBuf::from_str("hey/abc.py").unwrap()),
        Some("python3 hey/abc.py".into())
    );
}
