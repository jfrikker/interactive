use command::Command;
use escape::split_command;
use linefeed::{DefaultTerminal, Reader, ReadResult, Terminal};
use std::io;
use std::process;

pub struct Shell<T: Terminal> {
    reader: Reader<T>,
    cmd: Command
}

impl Shell<DefaultTerminal> {
    pub fn new(cmd: Command) -> io::Result<Self> {
        let reader = Reader::new("interactive")?;

        let mut res = Shell {
            reader,
            cmd
        };

        res.update_command(|_| {});
        Ok(res)
    }
}

impl <T: Terminal> Shell<T> {
    pub fn run(mut self) {
        while let Ok(ReadResult::Input(input)) = self.reader.read_line() {
            self.handle_line(input);
        }
    }

    pub fn handle_line(&mut self, line: String) {
        let add_to_history = {
            let args = split_command(&line);
            self.execute(&args)
        };

        if add_to_history {
            self.reader.add_history(line);
        }
    }

    fn execute(&mut self, args: &[&str]) -> bool {
        match args.get(0).map(|arg| *arg) {
            None => {
                false
            },
            Some("-") => {
                self.remove_opts(&args[1..]);
                true
            },
            Some("+") => {
                self.add_opts(&args[1..]);
                true
            },
            Some("++") => {
                self.add_opt_arg(&args[1..]);
                true
            },
            _ => {
                match self.cmd.build_command(&args)
                    .stdin(process::Stdio::inherit())
                    .stdout(process::Stdio::inherit())
                    .spawn() {
                    Ok(mut child) => { child.wait().unwrap(); },
                    Err(e) => eprintln!("{}", e)
                }
                true
            }
        }
    }

    fn remove_opts(&mut self, args: &[&str]) {
        self.update_command(|cmd| {
            for opt in args.iter() {
                cmd.remove_opt(opt);
            }
        });
    }

    fn add_opts(&mut self, args: &[&str]) {
        self.update_command(|cmd| {
            for opt in args.iter() {
                cmd.add_opt(opt);
            }
        });
    }

    fn add_opt_arg(&mut self, args: &[&str]) {
        if args.len() == 2 {
            self.update_command(|cmd| {
                cmd.add_opt_arg(args.get(0).unwrap(), args.get(1).unwrap());
            });
        } else {
            eprintln!("Usage: ++ <option> <arg>");
        }
    }

    fn update_command<F>(&mut self, f: F)
        where F: FnOnce(&mut Command) {
        f(&mut self.cmd);
        self.reader.set_prompt(&format!("> {} ", self.cmd));
    }
}