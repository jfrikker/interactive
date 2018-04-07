extern crate itertools;
extern crate linefeed;

mod escape;

use itertools::Itertools;
use linefeed::{Reader, ReadResult};
use std::env::args_os;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::process;

fn main() {
    let cmd = Command::new(args_os().skip(1));
    let mut reader = Reader::new("my-application").unwrap();

    reader.set_history_size(10);
    reader.set_prompt(&format!("{} ", cmd));

    while let Ok(ReadResult::Input(input)) = reader.read_line() {
        {
            let rest = escape::split_command(&input);
            if rest.is_empty() {
                continue;
            }

            let mut child = cmd.build_command(&rest)
                .stdin(process::Stdio::inherit())
                .stdout(process::Stdio::inherit())
                .spawn()
                .unwrap();
            child.wait().unwrap();
        }

        reader.add_history(input);
    }

    println!();
}

struct Command {
    cmd: OsString,
    args: Vec<OsString>
}

impl Command {
    fn new<I>(mut args: I) -> Command
        where I: Iterator<Item=OsString> {
        let cmd = args.next().unwrap();
        let args = args.collect();
        Command {
            cmd,
            args
        }
    }

    fn cmdline(&self, rest: &[&str]) -> Vec<OsString> {
        let mut result =self.args.clone();
        result.extend(
            rest.iter()
                .map(|s| OsString::from(s))
        );
        result
    }

    fn build_command(&self, rest: &[&str]) -> process::Command {
        let mut cmd = process::Command::new(&self.cmd);
        cmd.args(self.cmdline(rest));
        cmd
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.args.is_empty() {
            self.cmd.to_str().unwrap().fmt(f)
        } else {
            write!(f, "{} {}", self.cmd.to_str().unwrap(), self.args.iter().map(|s| s.to_str().unwrap()).join(" "))
        }
    }
}