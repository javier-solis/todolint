use regex::Regex;
use std::fs;

fn main() {
    let filename = "src/main.rs";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let general_todo_re = create_general_todo_regex();
    let specific_todo_re = create_specific_todo_regex();

    for (line_number, line) in contents.lines().enumerate() {
        process_line(&line, line_number, &general_todo_re, &specific_todo_re);
    }
}

fn process_line(line: &str, line_number: usize, general_re: &Regex, specific_re: &Regex) {
    if let Some(general_cap) = general_re.captures(line) {
        let todo_comment = &general_cap[0];
        let todo_content = &general_cap["todo_content"];
        let comment_content = &general_cap["comment_content"];

        println!("'todo' on line {}:", line_number + 1);
        println!("\tFull text: {}", todo_comment);
        println!("\tTodo content: {}", todo_content);
        println!("\tComment content: {}", comment_content);

        // Match against the specific regex
        if specific_re.is_match(todo_content) {
            println!("\tIs Valid: True");

            if let Some(specific_cap) = specific_re.captures(todo_content) {
                let delimiter_types = ["parens", "braces", "brackets", "angles"];
                let matched_delimiters: Vec<(&str, &str)> = delimiter_types
                    .iter()
                    .filter_map(|&delimiter_type| {
                        specific_cap
                            .name(delimiter_type)
                            .map(|matched_content| (delimiter_type, matched_content.as_str()))
                    })
                    .collect();

                println!(
                    "\tDelimiters Found: {:?}",
                    matched_delimiters
                        .iter()
                        .map(|&(delimiter_type, _)| delimiter_type)
                        .collect::<Vec<_>>()
                );

                for (delimiter_type, delimiter_content) in matched_delimiters {
                    println!("\tContents of {}: {}", delimiter_type, delimiter_content);
                }
            }
        } else {
            println!("\tIs Valid: False");
        }
        println!("\n");
    }
}

fn create_general_todo_regex() -> Regex {
    let todo_prefix = r"//\s*todo\s*";
    let todo_content = r"(?<todo_content>.*?)";
    let colon_separator = r"\s*:\s*";
    let comment_content = r"(?<comment_content>.*)";

    Regex::new(&format!(
        r"{}{}{}{}",
        todo_prefix, todo_content, colon_separator, comment_content
    ))
    .unwrap()
}

fn create_specific_todo_regex() -> Regex {
    let keyword_pattern = r"[a-zA-Z0-9_-]+";

    let parens_pattern = format!(r"\((?<parens>{})\)", keyword_pattern);
    let braces_pattern = format!(r"\{{(?<braces>{})\}}", keyword_pattern);
    let brackets_pattern = format!(r"\[(?<brackets>{})\]", keyword_pattern);
    let angles_pattern = format!(r"<(?<angles>{})>", keyword_pattern);

    let delimiter_pattern = format!(
        r"(?:{}|{}|{}|{})",
        parens_pattern, braces_pattern, brackets_pattern, angles_pattern
    );

    Regex::new(&format!(r"^{}{{0,3}}$", delimiter_pattern)).unwrap()
}
