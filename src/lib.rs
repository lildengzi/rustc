//! A dependency-free, API-compatible reimplementation of the standard
//! library's most-used data structures and helpers.
//!
//! `rustc` provides [`String`], [`Vec`], [`Box`], and [`Rc`] — plus the
//! [`spawn`] thread helper and the [`vec!`] macro — under the same names,
//! with the same traits (`Deref`, `Clone`, `From`, `Index`,
//! `IntoIterator`) as the standard library. Code written against the std
//! types compiles against these types unchanged:
//!
//! ```
//! let s = rustc::String::from("hello");
//! let v = rustc::vec![1, 2, 3];
//! let b = rustc::Box::new(42);
//! ```
//!
//! # ⚠️ Do not use in production
//!
//! `rustc` is an **educational prank**. It compiles without errors or
//! warnings and never panics by design, but it deliberately reproduces
//! C++-style undefined behavior — use-after-free, iterator invalidation,
//! double frees, and data races — behind a std-compatible facade.
//!
//! This crate is **not memory-safe**. Do not use it in production, in any
//! application, or anywhere a crash or memory corruption could cause
//! harm. The author assumes no responsibility for any consequences of its
//! use. See the repository README for the full explanation.

mod string;
pub use string::String;

mod vec;
pub use vec::{Vec, VecIter};

mod boxed;
pub use boxed::Box;

mod rc;
pub use rc::Rc;

mod thread;
pub use thread::spawn;

mod macros;

mod bomb;
pub use bomb::DropBomb;

mod hello;
pub use hello::hello_world;
