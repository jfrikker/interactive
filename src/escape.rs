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