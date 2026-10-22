pub fn longest_word(text: &str) -> &str {
    let mut longest = "";
    let mut longest_length = 0;

    for word in text.split_whitespace() {
        let length = word.chars().count();
        if length >= longest_length {
            longest = word;
            longest_length = length;
        }
    }

    longest
}

pub fn summarize_longest_word(text: &str) -> String {
    let longest = longest_word(text);
    let length = longest.chars().count();

    format!("Longest word: {longest} ({length} characters)", )
}