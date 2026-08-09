use crate::CppString;

pub struct DropBomb {
    s: CppString,
    dangling: &'static str,
}

impl DropBomb {
    pub fn new(text: &str) -> Self {
        let s = CppString::from(text);
        let dangling = s.as_str();
        DropBomb { s, dangling }
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(&mut self.s as *mut CppString);
        }
        let len = self.dangling.len();
        for &b in self.dangling.as_bytes() {
            let _ = b;
        }
        println!("DropBomb: read dangling str ({len} bytes)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_a_static_str() {
        let bomb = DropBomb::new("boom");
        let d: &'static str = bomb.dangling;
        assert_eq!(d, "boom");
        std::mem::forget(bomb);
    }
}
