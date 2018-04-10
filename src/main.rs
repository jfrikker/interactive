extern crate itertools;
extern crate linefeed;

mod command;
mod escape;

use command::Command;
use linefeed::{Reader, ReadResult, Terminal};
use std::env::args_os;
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
        let mut add_to_history = true;

        {
            let rest = escape::split_command(&input);
            match rest.get(0).map(|arg| *arg) {
                None => {
                    add_to_history = false;
                },
                Some("-") => {
                    if rest.len() > 0 {
                        for opt in rest.iter().skip(1) {
                            cmd.remove_opt(opt);
                        }
                        set_prompt(&mut reader, &cmd);
                    } else {
                        eprintln!("Usage: - <option> [<option> ...]");
                    }
                },
                Some("+") => {
                    if rest.len() > 0 {
                        for opt in rest.iter().skip(1) {
                            cmd.add_opt(opt);
                        }
                        set_prompt(&mut reader, &cmd);
                    } else {
                        eprintln!("Usage: + <option> [<option> ...]");
                    }
                },
                Some("++") => {
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
                        Ok(mut child) => { child.wait().unwrap(); },
                        Err(e) => eprintln!("{}", e)
                    }
                }
            }
        }

        if add_to_history {
            reader.add_history(input);
        }
    }

    println!();
}

fn set_prompt<T>(reader: &mut Reader<T>, cmd: &Command)
    where T: Terminal {
    reader.set_prompt(&format!("> {} ", cmd));
}