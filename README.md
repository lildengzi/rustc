# rustcpp

![rustcpp](assets/images/image1.png)

*Write Rust, crash like C++. Compiles clean; runtime is a lottery.*

[中文版说明 →](README.zh-CN.md)

A dependency-free, API-compatible reimplementation of the standard library's
most-used types — `String`, `Vec`, `Box`, `Rc` — plus a `spawn` thread helper
and the `vec!` macro. Zero warnings, safe-looking surface:

```rust
use rustcpp::*;

let mut s = String::from("hello");
s.push_str(", world");
println!("{s}");

let v = vec![1, 2, 3];
for &x in &v {
    println!("{x}");
}
```

Everything compiles exactly like the std types, implements the same traits,
and behaves identically in ordinary use.

## ⚠️ Do not use this crate in production

This crate is an **educational prank**, and the paragraph above is the joke.
It compiles cleanly and never panics by design, but the runtime behavior is
**deliberately, genuinely undefined**:

- `String` hands out a `&'static str` into a buffer it later frees — hold it
  across a drop and you have a use-after-free.
- `Vec` iterators are not bound to the collection, so mutating while
  iterating compiles — and corrupts memory.
- `Box` is shallow-cloneable; dropping two clones frees the same pointer
  twice.
- `Rc` keeps its count in a non-atomic cell yet claims to be thread-safe;
  across threads the count silently corrupts.
- `spawn` erases closure lifetimes, so a thread can read a local the caller
  has already dropped.

These are **not simulations**. They are real C++-style undefined behavior
implemented with `unsafe`: expect garbage reads, wrong results, data races,
and `free(): double free detected` aborts. The author accepts no
responsibility for any consequence of using it. Run the `examples/`
binaries if you want to watch each trap fire.
