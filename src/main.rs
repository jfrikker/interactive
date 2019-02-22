#[macro_use] extern crate quick_error;
extern crate dirs;
extern crate itertools;
extern crate linefeed;
extern crate nix;
extern crate try_map;

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
    let mut shell = shell::Shell::new(cmd).unwrap();
    shell.enable_save_history();
    shell.run();

    println!();
}