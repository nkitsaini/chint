use anyhow::Context;
use difference::{Changeset, Difference};
use std::io::Write;

// Screenshot:
// https://raw.githubusercontent.com/johannhof/difference.rs/master/assets/github-style.png

pub fn print_diff(text1: &str, text2: &str) -> anyhow::Result<()> {
    let Changeset { diffs, .. } = Changeset::new(text1, text2, "\n");

    let mut t = term::stdout().context("Terminal missing?")?;

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                t.reset()?;
                writeln!(t, " {}", x)?;
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        t.fg(term::color::GREEN)?;
                        write!(t, "+")?;
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    t.fg(term::color::GREEN)?;
                                    write!(t, "{}", z)?;
                                    write!(t, " ")?;
                                }
                                Difference::Add(ref z) => {
                                    t.fg(term::color::WHITE)?;
                                    t.bg(term::color::GREEN)?;
                                    write!(t, "{}", z)?;
                                    t.reset()?;
                                    write!(t, " ")?;
                                }
                                _ => (),
                            }
                        }
                        writeln!(t, "")?;
                    }
                    _ => {
                        t.fg(term::color::BRIGHT_GREEN)?;
                        writeln!(t, "+{}", x)?;
                    }
                };
            }
            Difference::Rem(ref x) => {
                t.fg(term::color::RED)?;
                writeln!(t, "-{}", x)?;
            }
        }
    }
    t.reset()?;
    t.flush()?;
    Ok(())
}
