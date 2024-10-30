//TODO: this should all probably go in some kind of template module.

//TODO: this should probably be a method of some kind of template struct instead of just a free-floating function.
//      that way, we can force it to be called on itself within the constructor, disallowing an invalid template object to exist in the first place.
fn validate_template(template: &str) -> Result<(), Vec<String>> {
    let errs = template.lines().enumerate().fold(Vec::new(), |mut lines_acc, (line_number, line)| {
        let mut last = ' ';
        let mut opening_brace: Option<(char, usize)> = None;

        let line_errs = line.chars().enumerate().fold(Vec::new(), |mut chars_acc, (i, ch)| {
            if last == '\\' {
                last = ' ';
                return chars_acc;
            }
            match ch {
                '\\' => {
                    if opening_brace.is_some() {
                        chars_acc.push(format!(
                            "Escape charactor not allowed in template field name - {line_number}:{i}."
                        ));
                    }
                }
                '{' => {
                    if let Some((_, brace)) = opening_brace {
                        chars_acc.push(format!(
                            "Tried to open a template field at {line_number}:{i} but one was already open at {line_number}:{brace}."
                        ));
                    }
                    opening_brace = Some((ch, i));
                }
                '}' => {
                    if last == '{' {
                        chars_acc.push(format!(
                            "Template field with no name at {line_number}:{}.",
                            i - 1
                        ));
                    }

                    if opening_brace.take().is_none() {
                        chars_acc.push(format!(
                            "Found closing bracket at {line_number}:{i} but no template field was opened on the same line. Consider escaping it with a '\\'."
                        ));
                    }
                }
                _ => {}
            }

            last = ch;
            chars_acc
        });

        lines_acc.extend(line_errs);
        if let Some((_, brace)) = opening_brace {
            lines_acc.push(format!("Unclosed brace at {line_number}:{brace}."));
        }
        lines_acc
    });

    if errs.len() != 0 {
        Err(errs)
    } else {
        Ok(())
    }
}

fn apply_template(template: &str, content: &std::collections::HashMap<&str, &str>) -> String {
    let mut res = String::new();
    for (line_number, line) in template.lines().enumerate() {
        // Vec of ((start, end), string to replace the range with)
        let mut replacements: Vec<((usize, usize), &str)> = vec![];

        let mut last = ' ';
        let mut opening_brace_index: usize = 0;

        for (i, ch) in line.chars().enumerate() {
            if last == '\\' {
                last = ' ';
                continue;
            }
            match ch {
                '\\' => {
                    replacements.push(((i, i), ""));
                }
                '{' => {
                    opening_brace_index = i;
                }
                '}' => {
                    // if a template is validated first, then we should only ever see opening_brace_index = the correct opening brace for this closing brace,
                    //     but in case this function starts failing regression tests, check here first.
                    let j = opening_brace_index;

                    let replacement = content.get(&line[j + 1..i]).unwrap_or_else(|| {
                        //TODO: when this becomes a method of the template struct, add information about the filename
                        //TODO: when I implement a config file for the user to define fields, add a line to this error message about double checking the cfg file.
                        eprintln!("WARNING: template field name `{}` provided on line {}:{} of template `PATH/TO/TEMPLATE.html`, but no such field was found for PATH/TO/TEMPORARY_ERROR_FILENAME.md.", &line[j + 1..i], line_number, j);
                        &""});
                    replacements.push(((j, i), replacement));
                }
                _ => {}
            }

            last = ch;
        }

        // make replacements in reverse order so as to preserve the locations
        let mut new_line = line.to_owned();
        while let Some(((start, end), replacement)) = replacements.pop() {
            new_line.replace_range(start..end + 1, replacement);
        }

        res.push_str(&new_line);
        res.push('\n');
    }
    res
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

        let result = apply_template(&template, &content);
        assert_eq!(result, "<html>content</html>\n".to_string());
    }

    #[test]
    fn escaped_template() {
        let template = r"<html>{body}\{test\}</html>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, &content);
        assert_eq!(result, "<html>content{test}</html>\n".to_string());
    }

    #[test]
    fn handle_all_escapes() {
        let template = r"<h\tml>{body}</htm\l>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, &content);
        let mut left = r"<html>content</html>".to_string();
        left.push('\n');
        assert_eq!(left, result);
    }

    #[test]
    fn ignore_nonexistent_fields_and_warn() {
        let template = r"<h\tml>{body}{nonsense}</htm\l>";
        let mut content = HashMap::new();
        content.insert("body", "content");

        let result = apply_template(&template, &content);
        let mut left = r"<html>content</html>".to_string();
        left.push('\n');
        assert_eq!(left, result);
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_opening_brace() {
        let template = r"<html>body}</html>";
        validate_template(&template).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_closing_brace() {
        let template = r"<html>{body</html>";
        validate_template(&template).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_no_template_field_name() {
        let template = r"<html>{}</html>";
        validate_template(&template).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_template_double_open() {
        let template = r"<html>{{}}</html>";
        validate_template(&template).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_escape_inside_field_name() {
        let template = r"<html>\\n{\body}\\n</html>";
        validate_template(&template).unwrap();
    }
}
