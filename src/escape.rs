pub fn escape(arg: &mut String) {
    if arg.contains(' ') || arg.contains('\'')  {
        *arg = arg.replace('\'', r#"'\''"#);
        arg.insert(0, '\'');
        arg.push('\'');
    }
}

pub fn split_command(line: &str) -> Split<'_> {
    Split {
        rest: line,
        state: ParseState::Normal,
        finished: false
    }
}

pub struct Split<'a> {
    rest: &'a str,
    state: ParseState,
    finished: bool
}

impl <'a> Split<'a> {
    fn capture(&mut self, i: usize, allow_empty: bool) -> Option<&'a str> {
        let res = if allow_empty || i > 0 {
            Some(&self.rest[..i])
        } else {
            None
        };
        self.rest = &self.rest[(i + 1)..];
        res
    }

    fn capture_continue(&mut self, i: usize, allow_empty: bool) -> Option<&'a str> {
        self.capture(i, allow_empty)
            .or_else(|| self.next())
    }

    fn capture_rest(&mut self, allow_empty: bool) -> Option<&'a str> {
        let res = if allow_empty || !self.rest.is_empty() {
            Some(self.rest)
        } else {
            None
        };
        self.finished = true;
        res
    }

    fn capture_until(&mut self, end: char) -> Option<&'a str> {
        for (i, c) in self.rest.chars().enumerate() {
            if c == end {
                self.state = ParseState::Normal;
                return self.capture_continue(i, true);
            }
        }

        self.capture_rest(true)
    }
}

impl <'a> Iterator for Split<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None
        }

        match self.state {
            ParseState::SingleQuote => return self.capture_until('\''),
            ParseState::DoubleQuote => return self.capture_until('"'),
            ParseState::Normal => for (i, c) in self.rest.chars().enumerate() {
                match c {
                    ' ' => return self.capture_continue(i, false),
                    '\'' => {
                        self.state = ParseState::SingleQuote;
                        return self.capture_continue(i, false)
                    },
                    '"' => {
                        self.state = ParseState::DoubleQuote;
                        return self.capture_continue(i, false)
                    }
                    _ => {}
                }
            }
        }

        self.capture_rest(false)
    }
}

enum ParseState {
    Normal,
    SingleQuote,
    DoubleQuote
}

#[cfg(test)]
mod tests {
    use super::{escape, split_command};

    fn test_parse(line: &str, expected: Vec<&str>) {
        let actual: Vec<&str> = split_command(line).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn empty() {
        test_parse("", vec!());
    }

    #[test]
    fn just_spaces() {
        test_parse("  ", vec!());
    }

    #[test]
    fn single_string() {
        test_parse("abc", vec!("abc"));
    }

    #[test]
    fn leading_space() {
        test_parse(" abc", vec!("abc"));
    }

    #[test]
    fn trailing_space() {
        test_parse("abc ", vec!("abc"));
    }

    #[test]
    fn two_args() {
        test_parse("abc def", vec!("abc", "def"));
    }

    #[test]
    fn two_args_more_space() {
        test_parse("abc    def", vec!("abc", "def"));
    }

    #[test]
    fn single_quote() {
        test_parse("one 'two three' four", vec!("one", "two three", "four"));
    }

    #[test]
    fn double_quote() {
        test_parse("one \"two three\" four", vec!("one", "two three", "four"));
    }

    #[test]
    fn double_in_single() {
        test_parse("one 'two\"three' four", vec!("one", "two\"three", "four"));
    }

    #[test]
    fn single_in_double() {
        test_parse("one \"two'three\" four", vec!("one", "two'three", "four"));
    }

    #[test]
    fn dangling_single() {
        test_parse("one 'two three", vec!("one", "two three"));
    }

    #[test]
    fn dangling_double() {
        test_parse("one \"two three", vec!("one", "two three"));
    }

    #[test]
    fn adjacent_quotes() {
        test_parse("one'two'\"three\"'four five'", vec!("one", "two", "three", "four five"));
    }

    fn test_escape(input: &str, expected: &str) {
        let mut s = input.into();
        escape(&mut s);
        assert_eq!(&s, expected);
    }

    #[test]
    fn escape_normal() {
        test_escape("abc", "abc");
    }

    #[test]
    fn escape_space() {
        test_escape("abc def", "'abc def'");
    }

    #[test]
    fn escape_quote() {
        test_escape(r#"abc'def"#, r#"'abc'\''def'"#);
    }
}