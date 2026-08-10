/// Creates a [`Vec`] containing the given elements, compatible with the
/// standard `vec!` macro.
///
/// # Examples
///
/// ```
/// let v = rustcpp::vec![1, 2, 3];
/// assert_eq!(v.len(), 3);
/// assert_eq!(v[2], 3);
/// ```
#[macro_export]
macro_rules! vec {
    ($($elem:expr),* $(,)?) => {{
        let mut v = $crate::Vec::new();
        $(v.push($elem);)*
        v
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn macro_expands_to_vec() {
        let v = crate::vec![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn macro_accepts_trailing_comma() {
        let v = crate::vec!["a", "b",];
        assert_eq!(v.len(), 2);
        assert_eq!(v[1], "b");
    }
}
