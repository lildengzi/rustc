#[macro_export]
macro_rules! cpp_vec {
    ($($elem:expr),* $(,)?) => {{
        let mut v = $crate::CppVec::new();
        $(v.push($elem);)*
        v
    }};
}

#[cfg(test)]
mod tests {
    use crate::CppVec;

    #[test]
    fn macro_expands_to_cppvec() {
        let v = crate::cpp_vec![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn macro_accepts_trailing_comma() {
        let v = crate::cpp_vec!["a", "b",];
        assert_eq!(v.len(), 2);
        assert_eq!(v[1], "b");
    }
}
