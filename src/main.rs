use regex::Regex;
use std::fs;

fn main() {
    let filename = "src/main.rs";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    // Regex to match any line that contains `// todo ... :`
    let general_todo_re = Regex::new(r"//\s*todo.*:").unwrap();

    // Parse through file to match against the general regex
    for (line_number, line) in contents.lines().enumerate() {
        if let Some(general_cap) = general_todo_re.captures(line) {
            let todo_comment = &general_cap[0];
            println!("'todo' on line {}:", line_number + 1);
            println!("\tFull text: {}", todo_comment);
            println!("\n");
        }
    }
}
