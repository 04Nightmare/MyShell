pub fn input_line_parsing(input: &str) -> Vec<String> {
    let mut result_args = Vec::new();
    let mut current_arg_buffer = String::new();
    let mut in_squote = false;
    let mut in_dquote = false;
    let mut is_escaped = false;

    let mut char_peek_iterator = input.chars().peekable();
    while let Some(current_char) = char_peek_iterator.next() {
        if is_escaped {
            current_arg_buffer.push(current_char);
            is_escaped = false;
        } else if current_char == '\\' {
            if in_squote {
                current_arg_buffer.push(current_char);
            } else if in_dquote {
                if let Some(&next_char) = char_peek_iterator.peek() {
                    if next_char == '"' || next_char == '\\' {
                        is_escaped = true;
                    } else {
                        current_arg_buffer.push(current_char);
                    }
                } else {
                    current_arg_buffer.push(current_char);
                }
            } else {
                is_escaped = true;
            }
        } else if current_char == '\'' && !in_dquote {
            in_squote = !in_squote;
        } else if current_char == '"' && !in_squote {
            in_dquote = !in_dquote;
        } else if current_char == ' ' && !in_squote && !in_dquote {
            if !current_arg_buffer.is_empty() {
                result_args.push(current_arg_buffer.clone());
                current_arg_buffer.clear();
            }
        } else {
            current_arg_buffer.push(current_char);
        }
    }
    if !current_arg_buffer.is_empty() {
        result_args.push(current_arg_buffer);
    }
    return result_args;
}
