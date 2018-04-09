pub fn split_command(line: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0 as usize;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for (i, c) in line.chars().enumerate() {
        if in_single_quote {
            match c {
                '\'' => {
                    result.push(&line[start..i]);
                    start = i + 1;
                    in_single_quote = false;
                }
                _ => {}
            }
        } else if in_double_quote {
            match c {
                '"' => {
                    result.push(&line[start..i]);
                    start = i + 1;
                    in_double_quote = false;
                }
                _ => {}
            }
        } else {
            match c {
                ' ' => {
                    if start < i {
                        result.push(&line[start..i]);
                    }
                    start = i + 1;
                },
                '\'' => {
                    if start < i {
                        result.push(&line[start..i]);
                    }
                    start = i + 1;
                    in_single_quote = true;
                },
                '"' => {
                    if start < i {
                        result.push(&line[start..i]);
                    }
                    start = i + 1;
                    in_double_quote = true;
                }
                _ => {}
            }
        }
    }

    if start < line.len() {
        result.push(&line[start .. ]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::split_command;

    #[test]
    fn empty() {
        assert_eq!(split_command(""), Vec::<&str>::new());
    }

    #[test]
    fn just_spaces() {
        assert_eq!(split_command("  "), Vec::<&str>::new());
    }

    #[test]
    fn single_string() {
        assert_eq!(split_command("abc"), vec!("abc"));
    }

    #[test]
    fn leading_space() {
        assert_eq!(split_command(" abc"), vec!("abc"));
    }

    #[test]
    fn trailing_space() {
        assert_eq!(split_command("abc "), vec!("abc"));
    }

    #[test]
    fn two_args() {
        assert_eq!(split_command("abc def"), vec!("abc", "def"));
    }

    #[test]
    fn two_args_more_space() {
        assert_eq!(split_command("abc    def"), vec!("abc", "def"));
    }

    #[test]
    fn single_quote() {
        assert_eq!(split_command("one 'two three' four"), vec!("one", "two three", "four"));
    }

    #[test]
    fn double_quote() {
        assert_eq!(split_command("one \"two three\" four"), vec!("one", "two three", "four"));
    }

    #[test]
    fn double_in_single() {
        assert_eq!(split_command("one 'two\"three' four"), vec!("one", "two\"three", "four"));
    }

    #[test]
    fn single_in_double() {
        assert_eq!(split_command("one \"two'three\" four"), vec!("one", "two'three", "four"));
    }

    #[test]
    fn dangling_single() {
        assert_eq!(split_command("one 'two three"), vec!("one", "two three"));
    }

    #[test]
    fn dangling_double() {
        assert_eq!(split_command("one \"two three"), vec!("one", "two three"));
    }

    #[test]
    fn adjacent_quotes() {
        assert_eq!(split_command("one'two'\"three\"'four five'"), vec!("one", "two", "three", "four five"));
    }
}