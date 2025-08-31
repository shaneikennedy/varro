// TODO consider phf crate for O(1) lookups if this grows or sucks
const STOP_WORDS: [&str; 10] = ["the", "and", "is", "in", "at", "of", "to", "a", "an", "for"];
pub fn tokenize(contents: &str) -> impl Iterator<Item = String> {
    contents.split_whitespace().filter_map(|w| {
        if !STOP_WORDS.contains(&w.to_lowercase().as_str()) {
            Some(w.to_lowercase())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tokenize_tests {
    use super::*;

    #[test]
    fn test_tokenize_lower_cases() {
        let contents = "smAll sIlly kitTy Cat".to_string();
        let tokens: Vec<String> = tokenize(&contents).collect();
        assert_eq!(vec!["small", "silly", "kitty", "cat"], tokens);
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let contents = "For once and for all".to_string();
        let tokens: Vec<String> = tokenize(&contents).collect();
        assert_eq!(vec!["once", "all"], tokens);
    }
}
