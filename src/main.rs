#[macro_use] extern crate quick_error;
extern crate itertools;
extern crate linefeed;

mod command;
mod escape;
mod history;
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
    let mut shell = shell::Shell::new(cmd).unwrap();
    shell.enable_save_history();
    shell.run();

    println!();
}