pub fn strip_message_counts(title: String) -> String {
    let start_count = regex::Regex::new("^\\(\\d+\\) *").unwrap();
    let end_count = regex::Regex::new(" *\\(\\d+\\)$").unwrap();
    end_count.replace(&start_count.replace(&title, "").into_owned(), "").into_owned()
}
