# `rustc` Prank Crate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `rustc` crate — a std-mimicking library whose safe-looking public API hides real C++-style undefined behavior (dangling refs, iterator invalidation, double free, data races), with deterministic unit tests where possible and crashing examples for the rest.

**Architecture:** A lib crate (`src/lib.rs`) re-exporting seven modules — `string`, `vec`, `boxed`, `rc`, `thread`, `bomb`, `macros`. Every public item is a safe `fn`/`struct`; all `unsafe` lives inside method bodies. Each trap type implements std-style traits (`Deref`, `Clone`, `From`, `Index`) so consumers write idiomatic-looking code. Six `examples/*.rs` binaries demonstrate each crash.

**Tech Stack:** Rust edition 2024, std only, zero external dependencies. Cargo test for the deterministic assertions.

## Global Constraints

- Package name `rustc`, version `0.1.0`, edition `2024`, no dependencies.
- Zero compiler warnings across lib, tests, and examples (`cargo build --all-targets`).
- Public API must contain **no `unsafe fn`** and no `unsafe` exposure; all `unsafe` is inside safe `fn` bodies.
- UB must be real (transmute, raw pointers, drop_in_place) — never replaced by a panic in production code. The only panics live in `#[cfg(test)]` guard types used to observe double-drop.
- Unit tests must not crash deterministically; UAF/segfault demos live in `examples/`.
- Public names: `CppString`, `CppVec`, `CppVecIter`, `CppBox`, `CppRc`, `spawn_cpp_dangerous`, `cpp_vec!`, `DropBomb`.

---

### Task 1: Scaffold library crate

**Files:**
- Delete: `src/main.rs`
- Create: `src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `src/lib.rs` (empty public surface), a `rustc` lib that `cargo test`/`cargo build` succeed on. Later tasks append `mod`/`pub use` lines to `src/lib.rs`.

- [ ] **Step 1: Delete the hello-world binary**

```bash
rm src/main.rs
```

- [ ] **Step 2: Create `src/lib.rs`**

```rust
//! rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。
```

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: builds the `rustc` lib, tests pass (empty suite).

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: scaffold rustc library crate"
```

---

### Task 2: CppString (dangling `'static` str)

**Files:**
- Create: `src/string.rs`
- Modify: `src/lib.rs` (add `mod string;` + `pub use string::CppString;`)
- Test: `#[cfg(test)] mod tests` inside `src/string.rs`

**Interfaces:**
- Consumes: `src/lib.rs` from Task 1.
- Produces:
  - `pub struct CppString` with `pub fn new() -> Self`, `pub fn as_str(&self) -> &'static str`, `pub fn len(&self) -> usize`, `pub fn as_bytes(&self) -> &[u8]`, `pub fn push_str(&mut self, s: &str)`.
  - `impl Deref<Target = str>`, `impl From<&str>`, `impl From<String>`, `impl Clone` (deep), `impl Display`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_is_static_and_consistent() {
        let s = CppString::from("hello");
        let d: &'static str = s.as_str();
        assert_eq!(d, "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn deref_to_str() {
        let s = CppString::from("abc");
        let upper: String = s.to_uppercase();
        assert_eq!(upper, "ABC");
    }

    #[test]
    fn push_str_appends() {
        let mut s = CppString::new();
        s.push_str("foo");
        s.push_str("bar");
        assert_eq!(s.as_str(), "foobar");
        assert_eq!(s.as_bytes(), b"foobar");
    }

    #[test]
    fn clone_is_deep() {
        let a = CppString::from("x");
        let mut b = a.clone();
        b.push_str("y");
        assert_eq!(a.as_str(), "x");
        assert_eq!(b.as_str(), "xy");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `CppString` not found.

- [ ] **Step 3: Write the implementation**

```rust
use std::fmt;
use std::ops::Deref;

pub struct CppString {
    inner: Vec<u8>,
}

impl CppString {
    pub fn new() -> Self {
        CppString { inner: Vec::new() }
    }

    pub fn as_str(&self) -> &'static str {
        let s: &str = unsafe { std::str::from_utf8_unchecked(&self.inner) };
        unsafe { std::mem::transmute::<&str, &'static str>(s) }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }
}

impl Deref for CppString {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }
}

impl From<&str> for CppString {
    fn from(s: &str) -> Self {
        CppString {
            inner: s.as_bytes().to_vec(),
        }
    }
}

impl From<String> for CppString {
    fn from(s: String) -> Self {
        CppString {
            inner: s.into_bytes(),
        }
    }
}

impl Clone for CppString {
    fn clone(&self) -> Self {
        CppString {
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Display for CppString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append to `src/lib.rs`:

```rust
mod string;
pub use string::CppString;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add src/string.rs src/lib.rs
git commit -m "feat: add CppString with dangling 'static as_str"
```

---

### Task 3: CppVec + CppVecIter (iterator invalidation)

**Files:**
- Create: `src/vec.rs`
- Modify: `src/lib.rs` (add `mod vec;` + `pub use vec::{CppVec, CppVecIter};`)
- Test: `#[cfg(test)] mod tests` inside `src/vec.rs`

**Interfaces:**
- Consumes: `src/lib.rs` from Task 1.
- Produces:
  - `pub struct CppVec<T>` with `pub fn new() -> Self`, `pub fn push(&mut self, v: T)`, `pub fn len(&self) -> usize`, `pub fn iter(&self) -> CppVecIter<T>`.
  - `pub struct CppVecIter<T>` implementing `Iterator<Item = &'static T>` — holds raw pointers, **no borrow of the vec**.
  - `impl<T> From<Vec<T>>`, `impl<T> Index<usize>`, `impl<'a, T> IntoIterator for &'a CppVec<T>` (also yielding `&'static T`).

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_index() {
        let mut v = CppVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn from_vec_and_iter() {
        let v = CppVec::from(vec![10, 20, 30]);
        let mut sum = 0;
        for &x in &v {
            sum += x;
        }
        assert_eq!(sum, 60);
    }

    #[test]
    fn push_inside_for_loop_compiles() {
        let mut v = CppVec::from(vec![1, 2, 3]);
        for &x in &v {
            if x == 1 {
                v.push(99);
                break;
            }
        }
        assert_eq!(v.len(), 4);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `CppVec` not found.

- [ ] **Step 3: Write the implementation**

```rust
use std::ops::Index;

pub struct CppVec<T> {
    inner: Vec<T>,
}

pub struct CppVecIter<T> {
    ptr: *const T,
    end: *const T,
}

impl<T> CppVecIter<T> {
    fn new(v: &CppVec<T>) -> Self {
        let ptr = v.inner.as_ptr();
        let end = unsafe { ptr.add(v.inner.len()) };
        CppVecIter { ptr, end }
    }
}

impl<T> Iterator for CppVecIter<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<&'static T> {
        if self.ptr == self.end {
            return None;
        }
        unsafe {
            let item = &*self.ptr;
            self.ptr = self.ptr.add(1);
            Some(std::mem::transmute::<&T, &'static T>(item))
        }
    }
}

impl<T> CppVec<T> {
    pub fn new() -> Self {
        CppVec { inner: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> CppVecIter<T> {
        CppVecIter::new(self)
    }
}

impl<T> From<Vec<T>> for CppVec<T> {
    fn from(inner: Vec<T>) -> Self {
        CppVec { inner }
    }
}

impl<T> Index<usize> for CppVec<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.inner[i]
    }
}

impl<'a, T> IntoIterator for &'a CppVec<T> {
    type Item = &'static T;
    type IntoIter = CppVecIter<T>;

    fn into_iter(self) -> CppVecIter<T> {
        CppVecIter::new(self)
    }
}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod vec;
pub use vec::{CppVec, CppVecIter};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (3 tests, including `push_inside_for_loop_compiles` which proves the no-borrow trap compiles).

- [ ] **Step 6: Commit**

```bash
git add src/vec.rs src/lib.rs
git commit -m "feat: add CppVec with unborrowed iterator"
```

---

### Task 4: CppBox (shallow clone + double free)

**Files:**
- Create: `src/boxed.rs`
- Modify: `src/lib.rs` (add `mod boxed;` + `pub use boxed::CppBox;`)
- Test: `#[cfg(test)] mod tests` inside `src/boxed.rs`

**Interfaces:**
- Consumes: `src/lib.rs` from Task 1.
- Produces:
  - `pub struct CppBox<T>` with `pub fn new(value: T) -> Self`.
  - `impl<T> Clone` (copies raw pointer, **no refcount**), `impl<T> Drop` (`Box::from_raw` → double free), `impl<T> Deref`, `impl<T> DerefMut`, `impl<T> From<T>`.
  - Test-only guard `PanicOnDoubleDrop` (thread_local `Cell` flag) whose `Drop` panics on the second invocation — this is what makes the double-free observable deterministically.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static DROP_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    struct PanicOnDoubleDrop;

    impl Drop for PanicOnDoubleDrop {
        fn drop(&mut self) {
            DROP_COUNT.with(|c| c.set(c.get() + 1));
            if DROP_COUNT.with(|c| c.get()) > 1 {
                panic!("CppBox double free");
            }
        }
    }

    #[test]
    fn deref_and_deref_mut() {
        let mut b = CppBox::new(5);
        *b += 1;
        assert_eq!(*b, 6);
    }

    #[test]
    fn from_trait() {
        let b: CppBox<i32> = 42.into();
        assert_eq!(*b, 42);
    }

    #[test]
    #[should_panic(expected = "CppBox double free")]
    fn shallow_clone_double_frees() {
        DROP_COUNT.with(|c| c.set(0));
        let a = CppBox::new(PanicOnDoubleDrop);
        let b = a.clone();
        drop(a);
        drop(b);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `CppBox` not found.

- [ ] **Step 3: Write the implementation**

```rust
use std::ops::{Deref, DerefMut};

pub struct CppBox<T> {
    ptr: *mut T,
}

impl<T> CppBox<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        CppBox { ptr }
    }
}

impl<T> From<T> for CppBox<T> {
    fn from(value: T) -> Self {
        CppBox::new(value)
    }
}

impl<T> Clone for CppBox<T> {
    fn clone(&self) -> Self {
        CppBox { ptr: self.ptr }
    }
}

impl<T> Deref for CppBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for CppBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for CppBox<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr));
        }
    }
}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod boxed;
pub use boxed::CppBox;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS. The `shallow_clone_double_frees` test panics in the test-only guard's `Drop` on the second drop and is caught by `#[should_panic]`. (The double `Box::from_raw` is real UB, but the guard's thread_local state makes the observation deterministic.)

- [ ] **Step 6: Commit**

```bash
git add src/boxed.rs src/lib.rs
git commit -m "feat: add CppBox with shallow clone and double free"
```

---

### Task 5: CppRc (non-atomic Cell refcount + false Send/Sync)

**Files:**
- Create: `src/rc.rs`
- Modify: `src/lib.rs` (add `mod rc;` + `pub use rc::CppRc;`)
- Test: `#[cfg(test)] mod tests` inside `src/rc.rs`

**Interfaces:**
- Consumes: `src/lib.rs` from Task 1.
- Produces:
  - `pub struct CppRc<T>` with `pub fn new(value: T) -> Self` and `pub fn strong_count(this: &Self) -> usize`.
  - `impl<T> Clone` (increments `Cell<usize>` non-atomically), `impl<T> Drop` (decrements; frees at 0), `impl<T> Deref`, `impl<T> From<T>`.
  - `unsafe impl<T: Send> Send`, `unsafe impl<T: Sync + Send> Sync` — **the false claim** that makes the data race reachable across threads.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn refcount_logic() {
        let a = CppRc::new(7);
        assert_eq!(CppRc::strong_count(&a), 1);
        let b = a.clone();
        assert_eq!(CppRc::strong_count(&a), 2);
        drop(b);
        assert_eq!(CppRc::strong_count(&a), 1);
        assert_eq!(*a, 7);
    }

    #[test]
    fn value_dropped_when_last_ref_dropped() {
        let drops = Rc::new(Cell::new(0usize));
        struct Track(usize, Rc<Cell<usize>>);
        impl Drop for Track {
            fn drop(&mut self) {
                self.1.set(self.1.get() + 1);
            }
        }
        let a = CppRc::new(Track(1, drops.clone()));
        let b = a.clone();
        drop(a);
        assert_eq!(drops.get(), 0);
        drop(b);
        assert_eq!(drops.get(), 1);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `CppRc` not found.

- [ ] **Step 3: Write the implementation**

```rust
use std::cell::Cell;
use std::ops::Deref;

struct RcInner<T> {
    value: T,
    count: Cell<usize>,
}

pub struct CppRc<T> {
    inner: *mut RcInner<T>,
}

impl<T> CppRc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::into_raw(Box::new(RcInner {
            value,
            count: Cell::new(1),
        }));
        CppRc { inner }
    }

    pub fn strong_count(this: &Self) -> usize {
        unsafe { (*this.inner).count.get() }
    }
}

impl<T> From<T> for CppRc<T> {
    fn from(value: T) -> Self {
        CppRc::new(value)
    }
}

impl<T> Clone for CppRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).count.set((*self.inner).count.get() + 1);
        }
        CppRc { inner: self.inner }
    }
}

impl<T> Deref for CppRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.inner).value }
    }
}

impl<T> Drop for CppRc<T> {
    fn drop(&mut self) {
        unsafe {
            let count = (*self.inner).count.get();
            if count == 1 {
                drop(Box::from_raw(self.inner));
            } else {
                (*self.inner).count.set(count - 1);
            }
        }
    }
}

unsafe impl<T: Send> Send for CppRc<T> {}
unsafe impl<T: Sync + Send> Sync for CppRc<T> {}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod rc;
pub use rc::CppRc;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (2 tests). The race itself is not observable single-threaded; `data_race` example (Task 10) demonstrates it.

- [ ] **Step 6: Commit**

```bash
git add src/rc.rs src/lib.rs
git commit -m "feat: add CppRc with non-atomic refcount and false Send/Sync"
```

---

### Task 6: spawn_cpp_dangerous (lifetime-erased thread spawn)

**Files:**
- Create: `src/thread.rs`
- Modify: `src/lib.rs` (add `mod thread;` + `pub use thread::spawn_cpp_dangerous;`)
- Test: `#[cfg(test)] mod tests` inside `src/thread.rs`

**Interfaces:**
- Consumes: `src/lib.rs` from Task 1.
- Produces: `pub fn spawn_cpp_dangerous<F>(f: F) -> std::thread::JoinHandle<()> where F: FnOnce()` — no `'static`/`Send` bounds on the public signature; internally unsizes to `Box<dyn FnOnce() + '_>` then `transmute`s to `Box<dyn FnOnce() + Send + 'static>` and spawns.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_and_joins() {
        let handle = spawn_cpp_dangerous(|| println!("hello from thread"));
        handle.join().unwrap();
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `spawn_cpp_dangerous` not found.

- [ ] **Step 3: Write the implementation**

```rust
pub fn spawn_cpp_dangerous<F>(f: F) -> std::thread::JoinHandle<()>
where
    F: FnOnce(),
{
    let boxed: Box<dyn FnOnce() + '_> = Box::new(f);
    let static_boxed: Box<dyn FnOnce() + Send + 'static> =
        unsafe { std::mem::transmute(boxed) };
    std::thread::spawn(move || static_boxed())
}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod thread;
pub use thread::spawn_cpp_dangerous;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (1 test). The dangling-capture scenario is demonstrated in the `thread_dangle` example (Task 10).

- [ ] **Step 6: Commit**

```bash
git add src/thread.rs src/lib.rs
git commit -m "feat: add spawn_cpp_dangerous with lifetime-erased closures"
```

---

### Task 7: cpp_vec! macro

**Files:**
- Create: `src/macros.rs`
- Modify: `src/lib.rs` (add `mod macros;`)
- Test: `#[cfg(test)] mod tests` inside `src/macros.rs`

**Interfaces:**
- Consumes: `CppVec` + `CppVec::new`/`push` from Task 3.
- Produces: `#[macro_export] macro_rules! cpp_vec` — exported at crate root as `rustc::cpp_vec!` (also `$crate::cpp_vec!`); expands to `CppVec::new()` + one `push` per element. **No** `pub use` re-export (avoids E0530 conflicts with `#[macro_export]`).

- [ ] **Step 1: Write the failing test**

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `crate::cpp_vec!` not found.

- [ ] **Step 3: Write the implementation**

```rust
#[macro_export]
macro_rules! cpp_vec {
    ($($elem:expr),* $(,)?) => {{
        let mut v = $crate::CppVec::new();
        $(v.push($elem);)*
        v
    }};
}
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod macros;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add src/macros.rs src/lib.rs
git commit -m "feat: add cpp_vec! macro"
```

---

### Task 8: DropBomb (reads dangling str in Drop)

**Files:**
- Create: `src/bomb.rs`
- Modify: `src/lib.rs` (add `mod bomb;` + `pub use bomb::DropBomb;`)
- Test: `#[cfg(test)] mod tests` inside `src/bomb.rs`

**Interfaces:**
- Consumes: `CppString::as_str` from Task 2.
- Produces: `pub struct DropBomb` with `pub fn new(text: &str) -> Self`. Holds a `CppString` plus its `&'static str`. `Drop` first `drop_in_place`s the `CppString` (freeing the buffer) then iterates the dangling str's bytes — a real UAF. The unit test only constructs and `mem::forget`s (never runs the UB `Drop`); the crash is demonstrated in the `bomb` example.

- [ ] **Step 1: Write the failing test**

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib`
Expected: FAIL to compile — `DropBomb` not found.

- [ ] **Step 3: Write the implementation**

```rust
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
```

- [ ] **Step 4: Wire into `src/lib.rs`**

Append:

```rust
mod bomb;
pub use bomb::DropBomb;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS (1 test). The `Drop` body's UAF read is not executed in the unit test (`mem::forget`); `examples/bomb.rs` (Task 10) demonstrates the crash.

- [ ] **Step 6: Commit**

```bash
git add src/bomb.rs src/lib.rs
git commit -m "feat: add DropBomb that reads dangling str in Drop"
```

---

### Task 9: README

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write the README**

```markdown
# rustc

rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。

> 这是一款教育 / 整活 crate：公开 API 模仿标准库，但内部用 `unsafe` 复刻
> C++ 的经典未定义行为（悬垂引用、迭代器失效、双重释放、数据竞争）。
> **千万不要在生产环境使用。**
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README"
```

---

### Task 10: Crash demo examples

**Files:**
- Create: `examples/uaf.rs`, `examples/iter_invalidate.rs`, `examples/double_free.rs`, `examples/data_race.rs`, `examples/thread_dangle.rs`, `examples/bomb.rs`

**Interfaces:**
- Consumes: the full public API from Tasks 2-8.

- [ ] **Step 1: Write `examples/uaf.rs`** (CppString use-after-free)

```rust
use rustc::CppString;

fn main() {
    let d: &'static str;
    {
        let s = CppString::from("hello, dangling world");
        d = s.as_str();
        println!("before drop: {d}");
    }
    println!("after drop:  {d}");
}
```

- [ ] **Step 2: Write `examples/iter_invalidate.rs`** (push during iteration)

```rust
use rustc::CppVec;

fn main() {
    let mut v = CppVec::from(vec![1, 2, 3]);
    for &x in &v {
        println!("item: {x}");
        if x == 2 {
            v.push(99);
            println!("pushed while iterating");
        }
    }
}
```

- [ ] **Step 3: Write `examples/double_free.rs`** (shallow-clone double free)

```rust
use rustc::CppBox;

fn main() {
    let a = CppBox::new(String::from("boom"));
    let b = a.clone();
    drop(a);
    println!("first drop done");
    drop(b);
    println!("unreachable");
}
```

- [ ] **Step 4: Write `examples/data_race.rs`** (non-atomic Cell refcount raced across threads)

```rust
use rustc::CppRc;
use std::thread;

fn main() {
    let rc = CppRc::new(0);
    let mut handles = Vec::new();
    for _ in 0..8 {
        let rc = rc.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100_000 {
                let _ = rc.clone();
                drop(rc.clone());
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("final count: {}", CppRc::strong_count(&rc));
}
```

- [ ] **Step 5: Write `examples/thread_dangle.rs`** (thread reads a dropped local)

```rust
use rustc::spawn_cpp_dangerous;
use std::thread;

fn main() {
    let message = String::from("thread says hi");
    let handle = spawn_cpp_dangerous(|| {
        thread::sleep(std::time::Duration::from_millis(10));
        println!("{message}");
    });
    drop(message);
    handle.join().unwrap();
}
```

- [ ] **Step 6: Write `examples/bomb.rs`** (DropBomb reads dangling str on drop)

```rust
use rustc::DropBomb;

fn main() {
    let _bomb = DropBomb::new("tick tock");
    println!("DropBomb armed");
}
```

- [ ] **Step 7: Build all examples**

Run: `cargo build --examples`
Expected: all 6 examples compile with zero warnings.

- [ ] **Step 8: Spot-run one example to confirm the trap fires**

Run: `cargo run --example uaf`
Expected: prints `before drop: hello, dangling world` then reads freed memory — either garbage or a crash. (UB output is nondeterministic by nature.)

- [ ] **Step 9: Commit**

```bash
git add examples/
git commit -m "feat: add crash demo examples"
```

---

### Task 11: Final verification

**Files:**
- None (verification only).

- [ ] **Step 1: Full test run**

Run: `cargo test`
Expected: all tests PASS (Tasks 2-8 suites; CppBox double-free test passes via `#[should_panic]`).

- [ ] **Step 2: Zero-warning check across all targets**

Run: `cargo build --all-targets 2>&1 | tee /dev/stderr | grep -c warning || true`
Expected: output `0`. If any warning appears, fix it (likely an unused item in an example or a missing `#![allow]`; resolve by removing the unused code rather than allowing).

- [ ] **Step 3: Final commit if anything changed**

```bash
git status --short && git add -A && git commit -m "chore: final verification fixes"
```

(If `git status` is clean, skip the commit.)
