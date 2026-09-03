/// Truncates at the largest char boundary ≤ `max_bytes`. Plain byte slicing
/// (`&s[..n]`) panics when `n` lands inside a multi-byte character — tool
/// output full of box-drawing chars or emoji hits that constantly.
pub fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::safe_truncate;

    #[test]
    fn boundary_safe_on_multibyte() {
        let s = "ab⚡cd"; // '⚡' is 3 bytes starting at index 2
        assert_eq!(safe_truncate(s, 3), "ab");
        assert_eq!(safe_truncate(s, 5), "ab⚡");
        assert_eq!(safe_truncate(s, 100), s);
    }
}
