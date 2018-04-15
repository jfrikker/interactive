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

        res.init();
        Ok(res)
    }
}

impl <T: Terminal> Shell<T> {
    #[allow(dead_code)]
    pub fn with_term(term: T, cmd: Command) -> io::Result<Self> {
        let reader = Reader::with_term("interactive", term)?;

        let mut res = Shell {
            reader,
            cmd
        };

        res.init();
        Ok(res)
    }

    fn init(&mut self) {
        self.update_command(|_| {});
    }

    pub fn run(mut self) {
        while let Ok(ReadResult::Input(input)) = self.reader.read_line() {
            self.handle_line(input);
        }
    }

    #[allow(dead_code)]
    pub fn get_cmd(&self) -> &Command {
        &self.cmd
    }

    pub fn handle_line(&mut self, line: String) {
        let add_to_history = {
            let args = split_command(&line);
            self.execute(&args).err().map(|e| eprintln!("{}", e));
            !args.is_empty()
        };

        if add_to_history {
            self.reader.add_history(line);
        }
    }

    #[allow(dead_code)]
    pub fn execute_line(&mut self, line: &str) -> Result<(), Error> {
        self.execute(&split_command(line))
    }

    fn execute(&mut self, args: &[&str]) -> Result<(), Error> {
        match args.get(0).map(|arg| *arg) {
            None => Ok(()),
            Some("-") => self.remove_opts(&args[1..]),
            Some("+") => self.add_opts(&args[1..]),
            Some("++") => self.add_opt_arg(&args[1..]),
            _ => {
                match self.cmd.build_command(&args)
                    .stdin(process::Stdio::inherit())
                    .stdout(process::Stdio::inherit())
                    .spawn() {
                    Ok(mut child) => { child.wait().unwrap(); },
                    Err(e) => eprintln!("{}", e)
                }
                Ok(())
            }
        }
    }

    fn remove_opts(&mut self, args: &[&str]) -> Result<(), Error> {
        self.update_command(|cmd| {
            for opt in args.iter() {
                cmd.remove_opt(opt);
            }
        });
        Ok(())
    }

    fn add_opts(&mut self, args: &[&str]) -> Result<(), Error> {
        self.update_command(|cmd| {
            for opt in args.iter() {
                cmd.add_opt(opt);
            }
        });
        Ok(())
    }

    fn add_opt_arg(&mut self, args: &[&str]) -> Result<(), Error> {
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
        self.reader.set_prompt(&format!("> {} ", self.cmd));
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