/// Returns the classic `"hello world"` greeting as a static string.
///
/// # Examples
///
/// ```
/// let d: &'static str = rustcpp::hello_world();
/// ```
pub fn hello_world() -> &'static str {
    let bytes = *b"hello world";
    let ptr = bytes.as_ptr();
    let dangling: &'static str = unsafe {
        let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, bytes.len()));
        std::mem::transmute::<&str, &'static str>(s)
    };
    dangling
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_forged_static_str() {
        let d: &'static str = hello_world();
        std::hint::black_box(d);
    }
}
