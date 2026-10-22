use std::collections::HashMap;

/// Counts the frequency of each word in the input text.
///
/// Returns a formatted string like:
/// "hello: 2, world: 1"
///
/// The order of words in the output does not matter.
///
/// # Example
/// ```
/// word_frequency("hello world hello"); // -> "hello: 2, world: 1"
/// ```
pub fn word_frequency(text: &str) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for word in text.split_whitespace() {
        *counts.entry(word.to_string()).or_insert(0) += 1;

    }

    counts
        .iter()
        .map(|(|word, count)| format!("{}: {}", word, count))
        .collect::<Vec<String>>()
        .join(", ")

}