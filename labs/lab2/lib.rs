pub fn is_palindrome(text: &str) -> bool {
    let lower = text.to_lowercase();
    
    let cleaned: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
    
    let reversed: String = cleaned.chars().rev().collect();

    reversed.chars().eq(cleaned.chars())
}

