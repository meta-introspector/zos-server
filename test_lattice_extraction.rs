use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let test_content = r#"
let gandalf = 71;
const WIZARD: i32 = 42;
let test = 999;
"#;

    println!("Testing value extraction:");
    println!("Content:\n{}", test_content);

    let mut values = Vec::new();

    // Extract literals and constants
    for line in test_content.lines() {
        println!("Processing line: '{}'", line);

        // Find numeric literals
        for word in line.split_whitespace() {
            println!("  Checking word: '{}'", word);

            // Try to parse as number, removing punctuation
            let clean_word = word.trim_end_matches(|c: char| !c.is_ascii_digit());
            if let Ok(num) = clean_word.parse::<i64>() {
                println!("    ✅ Found number: {}", num);
                values.push(num.to_string());
            } else {
                println!("    ❌ Not a number: '{}'", clean_word);
            }
        }
    }

    println!("\nExtracted values: {:?}", values);
}
