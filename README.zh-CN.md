# rustc（中文说明）

![rustc](assets/images/image1.png)

rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。

[English README →](README.md)

一个零依赖、与标准库 API 兼容的重新实现：`String`、`Vec`、`Box`、`Rc`，外加 `spawn` 线程辅助函数和 `vec!` 宏。零警告，安全的外表：

```rust
use rustc::*;

let mut s = String::from("hello");
s.push_str(", world");
println!("{s}");

let v = vec![1, 2, 3];
for &x in &v {
    println!("{x}");
}
```

一切照常编译、实现同样的 trait、日常使用行为完全一致。

## ⚠️ 千万不要在生产环境使用

这是一个**教育整活 crate**，上面那段是玩笑。它编译零错误零警告、从不 panic，但运行时是**刻意且真实的未定义行为**：

- `String` 会交出指向「未来会被释放缓冲区」的 `&'static str`——跨 drop 持有就是悬垂引用；
- `Vec` 的迭代器不绑定集合，边迭代边修改能编译通过——然后内存损坏；
- `Box` 是浅克隆，两个克隆同时 drop 就是双重释放；
- `Rc` 用非原子计数却自称线程安全，跨线程计数会悄悄坏掉；
- `spawn` 抹掉闭包生命周期，线程能读到一个已被调用方 drop 的局部变量。

**不是模拟，是真的**：垃圾读取、错误结果、数据竞争、`free(): double free detected` 崩溃，样样都有。作者对使用后果不承担任何责任。想围观每个陷阱爆炸，跑一遍 `examples/` 下的二进制即可。
