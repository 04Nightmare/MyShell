pub fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        let mut i = 0;
        let max = prefix.len().min(s.len());
        while i < max && prefix.as_bytes()[i] == s.as_bytes()[i] {
            i += 1;
        }
        prefix.truncate(i);
        if prefix.is_empty() {
            break;
        }
    }
    return prefix;
}
