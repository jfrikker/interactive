use command::Command;
use escape::split_command;
use itertools::Itertools;
use linefeed::{DefaultTerminal, Interface, ReadResult, Terminal};
use std::fs::{self, create_dir_all};
use nix::sys::signal;
use std::io;
use std::iter::Peekable;
use std::path::{PathBuf};
use std::process;
use dirs::home_dir;
use try_map::FallibleMapExt;

use std::os::unix::process::CommandExt;

pub struct Shell<T: Terminal> {
    reader: Interface<T>,
    cmd: Command,
    last_cmd: Option<String>,
    save_history: bool
}

impl Shell<DefaultTerminal> {
    pub fn new(cmd: Command) -> io::Result<Self> {
        Self::with_term(DefaultTerminal::new()?, cmd)
    }
}

impl <T: Terminal> Shell<T> {
    pub fn with_term(term: T, cmd: Command) -> io::Result<Self> {
        let reader = Interface::with_term("interactive", term)?;
        reader.set_history_size(1000);

        let mut res = Self {
            reader,
            cmd,
            last_cmd: None,
            save_history: false
        };

        res.init();
        Ok(res)
    }

    fn init(&mut self) {
        self.update_command(|_| {});
    }

    pub fn enable_save_history(&mut self) {
        if let Err(e) = self.read_history() {
            eprintln!("Error reading history: {}", e);
        }
        self.save_history = true;
    }

    fn read_history(&mut self) -> io::Result<()> {
        self.prepare_history_path()?.map_or(Ok(()), |path| self.reader.load_history(path))
    }

    pub fn run(mut self) {
        while let Ok(ReadResult::Input(input)) = self.reader.read_line() {
            self.handle_line(input);
        }
    }

    #[cfg(test)]
    pub fn get_cmd(&self) -> &Command {
        &self.cmd
    }

    pub fn handle_line(&mut self, line: String) {
        let add_to_history = {
            let mut args = split_command(&line).peekable();
            let empty = args.peek().is_none();
            if let Err(e) = self.execute(args) {
                eprintln!("{}", e);
            }
            !empty
        };

        if add_to_history {
            self.add_history(line);
        }
    }

    #[cfg(test)]
    pub fn execute_line(&mut self, line: &str) -> Result<(), Error> {
        self.execute(split_command(line).peekable())
    }

    fn execute<'a, I>(&mut self, mut args: Peekable<I>) -> Result<(), Error>
        where I: Iterator<Item=&'a str> {
        match args.peek().cloned() {
            None => Ok(()),
            Some("-") => self.remove_opts(args.skip(1)),
            Some("+") => self.add_opts(args.skip(1)),
            Some("++") => self.add_opt_arg(args.skip(1)),
            _ => {
                let old_sig = unsafe {
                    signal::signal(signal::Signal::SIGINT, signal::SigHandler::SigIgn).unwrap()
                };

                if let Err(e) = self.cmd.build_command(args)
                    .stdin(process::Stdio::inherit())
                    .stdout(process::Stdio::inherit())
                    .before_exec(|| {
                        unsafe {
                            signal::signal(signal::Signal::SIGINT, signal::SigHandler::SigDfl).unwrap();
                            Ok(())
                        }
                    })
                    .spawn()
                    .and_then(|mut child| child.wait()) {
                    eprintln!("{}", e);
                }
                
                unsafe {
                    signal::signal(signal::Signal::SIGINT, old_sig).unwrap();
                }

                Ok(())
            }
        }
    }

    fn remove_opts<'a, I>(&mut self, args: I) -> Result<(), Error>
        where I: IntoIterator<Item=&'a str> {
        self.update_command(|cmd| {
            for opt in args {
                cmd.remove_opt(opt);
            }
        });
        Ok(())
    }

    fn add_opts<'a, I>(&mut self, args: I) -> Result<(), Error>
        where I: IntoIterator<Item=&'a str> {
        self.update_command(|cmd| {
            for opt in args {
                cmd.add_opt(opt);
            }
        });
        Ok(())
    }

    fn add_opt_arg<'a, I>(&mut self, args: I) -> Result<(), Error>
        where I: IntoIterator<Item=&'a str> {
        let args = args.into_iter().collect_vec();
        if args.len() == 2 {
            self.update_command(|cmd| {
                cmd.add_opt_arg(args.get(0).unwrap(), args.get(1).unwrap());
            });
            Ok(())
        } else {
            Err(Error::Usage("++ <option> <arg>"))
        }
    }

    fn update_command<F>(&mut self, f: F)
        where F: FnOnce(&mut Command) {
        f(&mut self.cmd);
        self.reader.set_prompt(&format!("> {} ", self.cmd)).unwrap();
    }

    fn add_history(&mut self, line: String) {
        if Some(&line) != self.last_cmd.as_ref() {
            self.reader.add_history(line.clone());
            self.last_cmd = Some(line)
        }
    }

    fn prepare_history_path(&self) -> io::Result<Option<PathBuf>> {
        home_dir()
            .try_map(move |mut path| {
                path.push(".interactive");
                path.push("history");
                create_dir_all(&path)?;
                path.push(self.cmd.get_command());

                fs::OpenOptions::new().append(true).create(true).open(&path)?;
                Ok(path)
            })
    }
}

impl <T: Terminal> Drop for Shell<T> {
    fn drop(&mut self) {
        if self.save_history {
            if let Err(e) = self.prepare_history_path()
                .and_then(|optpath| optpath.map_or(Ok(()), |path| self.reader.save_history(path))) {
                eprintln!("\nError writing history: {}", e);
            }
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Usage(err: &'static str) {
            display("Usage: {}", err)
            description(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linefeed::memory::MemoryTerminal;
    use linefeed::terminal::Size;

    #[test]
    fn just_command() {
        let cmd = Command::new(vec!("cmd"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let shell = Shell::with_term(term, cmd).unwrap();
        assert_eq!(shell.get_cmd().to_string(), "cmd");
    }

    #[test]
    fn add_opts() {
        let cmd = Command::new(vec!("cmd"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let mut shell = Shell::with_term(term, cmd).unwrap();
        shell.handle_line("+ a b cde".to_string());
        assert_eq!(shell.get_cmd().to_string(), "cmd -a -b --cde");
    }

    #[test]
    fn remove_opts() {
        let cmd = Command::new(vec!("cmd", "-a", "-b", "arg", "--cde"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let mut shell = Shell::with_term(term, cmd).unwrap();
        shell.handle_line("- b cde".to_string());
        assert_eq!(shell.get_cmd().to_string(), "cmd -a");
    }

    #[test]
    fn add_opt_arg() {
        let cmd = Command::new(vec!("cmd"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let mut shell = Shell::with_term(term, cmd).unwrap();
        shell.handle_line("++ opt val".to_string());
        assert_eq!(shell.get_cmd().to_string(), "cmd --opt val");
    }

    #[test]
    fn add_opt_arg_too_few() {
        let cmd = Command::new(vec!("cmd"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let mut shell = Shell::with_term(term, cmd).unwrap();
        let res = shell.execute_line("++ opt");
        assert_eq!(res.expect_err("Expected error").to_string(), "Usage: ++ <option> <arg>");
    }

    #[test]
    fn add_opt_arg_too_many() {
        let cmd = Command::new(vec!("cmd"));
        let term = MemoryTerminal::with_size(Size{lines: 20, columns: 80});
        let mut shell = Shell::with_term(term, cmd).unwrap();
        let res = shell.execute_line("++ opt arg arg2");
        assert_eq!(res.expect_err("Expected error").to_string(), "Usage: ++ <option> <arg>");
    }
}