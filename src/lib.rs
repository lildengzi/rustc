//! rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。

mod string;
pub use string::CppString;

mod vec;
pub use vec::{CppVec, CppVecIter};

mod boxed;
pub use boxed::CppBox;

mod rc;
pub use rc::CppRc;

mod thread;
pub use thread::spawn_cpp_dangerous;

mod macros;

mod bomb;
pub use bomb::DropBomb;

mod hello;
pub use hello::hello_world;
