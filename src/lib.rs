//TODO: it may be good to use a separate function validate the templates first, then producing output from a template after without checking.
//      that way, we can
//          - panic before we output a single file
//          - limit the scope of the apply_template function or whatever we'll call it to the data transformations instead of mixing in lots of error handling cruft

fn apply_template(
    template: &str,
    content: std::collections::HashMap<&str, &str>,
) -> Result<String, String> {
    let mut res = String::new();
    for (line_number, line) in template.lines().enumerate() {
        // Vec of ((start, end), string to replace the range with)
        let mut replacements: Vec<((usize, usize), &str)> = vec![];

        let mut last = ' ';
        let mut opening_brace: Option<(char, usize)> = None;

        for (i, ch) in line.chars().enumerate() {
            if last == '\\' {
                last = ' ';
                continue;
            }
            match ch {
                '\\' => {
                    if opening_brace.is_some() {
                        return Err(format!(
                            "Escape charactor not allowed in template field name - {line_number}:{i}."
                        ));
                    }
                    replacements.push(((i, i), ""));
                }
                '{' => {
                    if let Some((_, brace)) = opening_brace {
                        return Err(format!(
                            "Tried to open a template field at {line_number}:{i} but one was already open at {line_number}:{brace}."
                        ));
                    }
                    opening_brace = Some((ch, i));
                }
                '}' => {
                    if last == '{' {
                        return Err(format!(
                            "Template field with no name at {line_number}:{}.",
                            i - 1
                        ));
                    }

                    if let Some((_, j)) = opening_brace.take() {
                        let replacement = content.get(&line[j + 1..i]).unwrap_or(&"");
                        replacements.push(((j, i), replacement));
                    } else {
                        return Err(format!(
                            "Found closing bracket at {line_number}:{i} but no template field was opened on the same line. Consider escaping it with a '\\'."
                        ));
                    }
                }
                _ => {}
            }

            last = ch;
        }

        if let Some((_, brace)) = opening_brace {
            return Err(format!("Unclosed brace at {line_number}:{brace}."));
        }

        // make replacements in reverse order so as to preserve the locations
        let mut new_line = line.to_owned();
        while let Some(((start, end), replacement)) = replacements.pop() {
            new_line.replace_range(start..end + 1, replacement);
        }

        res.push_str(&new_line);
        res.push('\n');
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn simple_template() {
        let template = "<html>{body}</html>\n";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, content);
        assert_eq!(result, Ok("<html>content</html>\n".to_string()));
    }

    #[test]
    fn escaped_template() {
        let template = r"<html>{body}\{test\}</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, content);
        assert_eq!(result, Ok("<html>content{test}</html>\n".to_string()));
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_opening_brace() {
        let template = r"<html>body}</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let _ = apply_template(&template, content).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_closing_brace() {
        let template = r"<html>{body</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let _ = apply_template(&template, content).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_template_field_name() {
        let template = r"<html>{}</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let _ = apply_template(&template, content).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_double_open() {
        let template = r"<html>{{}}</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let _ = apply_template(&template, content).unwrap();
    }

    #[test]
    #[should_panic]
    fn escape_inside_field_name() {
        let template = r"<html>\\n{\body}\\n</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let _ = apply_template(&template, content).unwrap();
    }

    #[test]
    fn handle_all_escapes() {
        let template = r"<h\tml>{body}</htm\l>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, content).unwrap();
        let mut left = r"<html>content</html>".to_string();
        left.push('\n');
        assert_eq!(left, result);
    }
}
