use escape::escape;
use itertools::Itertools;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::process;

pub struct Command {
    cmd: OsString,
    args: Vec<OsString>
}

impl Command {
    pub fn new<I, T>(args: I) -> Self
        where I: IntoIterator<Item=T>,
              T: Into<OsString> {
        let mut iter = args.into_iter();
        let cmd = iter.next().unwrap().into();
        let args = iter.map(Into::into).collect();
        Self {
            cmd,
            args
        }
    }

    pub fn get_command(&self) -> &OsStr {
        &self.cmd
    }

    fn cmdline<'a, I>(&self, rest: I) -> Vec<OsString>
        where I: IntoIterator<Item=&'a str> {
        let mut result = self.args.clone();
        result.extend(
            rest.into_iter()
                .map(OsString::from)
        );
        result
    }

    pub fn build_command<'a, I>(&self, rest: I) -> process::Command
        where I: IntoIterator<Item=&'a str> {
        let mut cmd = process::Command::new(&self.cmd);
        cmd.args(self.cmdline(rest));
        cmd
    }

    pub fn remove_opt(&mut self, opt: &str) {
        let single_opt = OsString::from(String::from("-") + opt.trim_matches('-'));
        let double_opt = OsString::from(String::from("--") + opt.trim_matches('-'));

        let mut found_arg = false;
        self.args.retain(|arg| {
            if found_arg {
                found_arg = false;

                if !arg.to_str().unwrap().starts_with('-') {
                    return false;
                }
            }

            if arg == &single_opt || arg == &double_opt {
                found_arg = true;
                false
            } else {
                true
            }
        });
    }

    pub fn add_opt(&mut self, opt: &str) {
        self.remove_opt(opt);

        let full_opt = if opt.starts_with('-') {
            OsString::from(opt)
        } else if opt.len() == 1 {
            OsString::from(String::from("-") + opt)
        } else {
            OsString::from(String::from("--") + opt)
        };

        self.args.push(full_opt);
    }

    pub fn add_opt_arg(&mut self, opt: &str, arg: &str) {
        self.add_opt(opt);
        self.args.push(OsString::from(arg));
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.is_empty() {
            self.cmd.to_str().unwrap().fmt(f)
        } else {
            write!(f, "{} {}",
                   self.cmd.to_str().unwrap(),
                   self.args.iter().map(|s| {
                       let mut arg = String::from(s.to_str().unwrap());
                       escape(&mut arg);
                       arg
                   }).join(" "))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn just_command() {
        let cmd = Command::new(vec!("cmd"));
        assert_eq!(cmd.to_string(), "cmd");
    }

    #[test]
    fn cmd_with_args() {
        let cmd = Command::new(vec!("cmd", "arg1", "arg2"));
        assert_eq!(cmd.to_string(), "cmd arg1 arg2");
    }

    #[test]
    fn add_single_flag() {
        let mut cmd = Command::new(vec!("cmd"));
        cmd.add_opt("t");
        assert_eq!(cmd.to_string(), "cmd -t");
    }

    #[test]
    fn add_multi_flag() {
        let mut cmd = Command::new(vec!("cmd"));
        cmd.add_opt("test");
        assert_eq!(cmd.to_string(), "cmd --test");
    }

    #[test]
    fn add_existing_flag() {
        let mut cmd = Command::new(vec!("cmd", "-t", "--t2"));
        cmd.add_opt("t");
        assert_eq!(cmd.to_string(), "cmd --t2 -t");
    }

    #[test]
    fn add_existing_flag_2() {
        let mut cmd = Command::new(vec!("cmd", "--t", "--t2"));
        cmd.add_opt("t");
        assert_eq!(cmd.to_string(), "cmd --t2 -t");
    }

    #[test]
    fn add_existing_flag_3() {
        let mut cmd = Command::new(vec!("cmd", "-t2", "-t"));
        cmd.add_opt("t2");
        assert_eq!(cmd.to_string(), "cmd -t --t2");
    }

    #[test]
    fn remove_flag() {
        let mut cmd = Command::new(vec!("cmd", "--t1", "--t2"));
        cmd.remove_opt("t1");
        assert_eq!(cmd.to_string(), "cmd --t2");
    }

    #[test]
    fn remove_flag_arg() {
        let mut cmd = Command::new(vec!("cmd", "--t1", "somearg", "--t2"));
        cmd.remove_opt("t1");
        assert_eq!(cmd.to_string(), "cmd --t2");
    }

    #[test]
    fn add_arg() {
        let mut cmd = Command::new(vec!("cmd"));
        cmd.add_opt_arg("t", "somearg");
        assert_eq!(cmd.to_string(), "cmd -t somearg");
    }

    #[test]
    fn remove_multi() {
        let mut cmd = Command::new(vec!("cmd", "-t", "-a", "-t", "-b", "-t"));
        cmd.remove_opt("t");
        assert_eq!(cmd.to_string(), "cmd -a -b");
    }
}