//! rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。

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
