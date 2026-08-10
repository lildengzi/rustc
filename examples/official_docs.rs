//! 照抄官方文档 / The Rust Book 的示例代码，`use rustc::*` 后照常编译、零警告、
//! 运行不 panic——然后在作用域结束那一刻被真实打飞（double free → abort）。
use rustc::*;

fn main() {
    // The Rust Book, Chapter 4 (Ownership):
    // "If we do want to deeply copy the heap data ... we can use a common
    // method called `clone`."
    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("s1 = {s1}, s2 = {s2}");
    // s2 与 s1 共享同一块堆内存。作用域结束，s2 drop → 释放一次；
    // s1 drop → 再释放一次。free(): double free detected → abort。
}
