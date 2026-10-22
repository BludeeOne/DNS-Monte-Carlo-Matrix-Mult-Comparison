pub fn word_stats(text: &str) -> String {

    let x = text.split_whitespace().count();
    let y = text.chars().count();
    format!("Words: {x}, Characters: {y}", )
}