extern crate itertools;
extern crate linefeed;

mod command;
mod escape;
mod shell;

use command::Command;
use std::env::args_os;
use std::process;

fn main() {
    if args_os().len() < 2 {
        eprintln!("Usage: interactive <command>");
        process::exit(1);
    }

    let cmd = Command::new(args_os().skip(1));
    let shell = shell::Shell::new(cmd).unwrap();
    shell.run();

    println!();
}