extern crate itertools;
extern crate linefeed;

mod escape;

use itertools::Itertools;
use linefeed::{Reader, ReadResult, Terminal};
use std::env::args_os;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::mem::swap;
use std::process;

fn main() {
    if args_os().len() < 2 {
        eprintln!("Usage: interactive <command>");
        process::exit(1);
    }

    let mut cmd = Command::new(args_os().skip(1));
    let mut reader = Reader::new("interactive").unwrap();

    reader.set_history_size(100);
    set_prompt(&mut reader, &cmd);

    while let Ok(ReadResult::Input(input)) = reader.read_line() {
        {
            let rest = escape::split_command(&input);
            if rest.is_empty() {
                continue;
            }

            match *rest.get(0).unwrap() {
                "-" => {
                    if rest.len() > 0 {
                        for opt in rest.iter().skip(1) {
                            cmd.remove_opt(opt);
                        }
                        set_prompt(&mut reader, &cmd);
                    } else {
                        eprintln!("Usage: - <option> [<option> ...]");
                    }
                },
                "+" => {
                    if rest.len() > 0 {
                        for opt in rest.iter().skip(1) {
                            cmd.add_opt(opt);
                        }
                        set_prompt(&mut reader, &cmd);
                    } else {
                        eprintln!("Usage: + <option> [<option> ...]");
                    }
                },
                "++" => {
                    if rest.len() == 3 {
                        cmd.add_opt_arg(rest.get(1).unwrap(), rest.get(2).unwrap());
                        set_prompt(&mut reader, &cmd);
                    } else {
                        eprintln!("Usage: ++ <option> <arg>");
                    }
                },
                _ => {
                    match cmd.build_command(&rest)
                        .stdin(process::Stdio::inherit())
                        .stdout(process::Stdio::inherit())
                        .spawn() {
                        Ok(mut child) => {child.wait().unwrap();},
                        Err(e) => eprintln!("{}", e)
                    }
                }
            }
        }

        reader.add_history(input);
    }

    println!();
}

fn set_prompt<T>(reader: &mut Reader<T>, cmd: &Command)
    where T: Terminal {
    reader.set_prompt(&format!("> {} ", cmd));
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

    fn remove_opt(&mut self, opt: &str) {
        let single_opt = OsString::from(String::from("-") + opt.trim_matches('-'));
        let double_opt = OsString::from(String::from("--") + opt.trim_matches('-'));
        let mut new_args = Vec::new();

        {
            let mut args = self.args.iter().peekable();
            loop {
                match args.next() {
                    None => break,
                    Some(arg) => {
                        if arg == &single_opt || arg == &double_opt {
                            let remove_next = match args.peek() {
                                Some(next_arg) => {
                                    !next_arg.to_str().unwrap().starts_with("-")
                                },
                                None => false
                            };

                            if remove_next {
                                args.next();
                            }
                        } else {
                            new_args.push(arg.clone());
                        }
                    }
                }
            }
        }

        swap(&mut self.args, &mut new_args);
    }

    fn add_opt(&mut self, opt: &str) {
        self.remove_opt(opt);

        let full_opt = if opt.starts_with("-") {
            OsString::from(opt)
        } else if opt.len() == 1 {
            OsString::from(String::from("-") + opt)
        } else {
            OsString::from(String::from("--") + opt)
        };

        self.args.push(full_opt);
    }

    fn add_opt_arg(&mut self, opt: &str, arg: &str) {
        self.add_opt(opt);
        self.args.push(OsString::from(arg));
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