# 设计：`rustc` —— 复刻 C++ 未定义行为的整活 crate

- 日期：2026-08-09
- 状态：已批准（Approach A：硬核原教旨 UB）

## 愿景

打造一个伪装成标准库增强品的 Rust crate（包名 `rustc`）。公开 API 全部模仿标准库
（`String`、`Vec`、`Box`、`Rc`、`thread::spawn`），实现 `Deref`、`Clone`、`From`、
`Index` 等常见 trait，文档示例一本正经。编译 100% 通过，公开 API 无任何 `unsafe`
（`unsafe` 全藏在库内部），无编译警告，但运行时稳定复刻 C++ 经典未定义行为。

## 约束

- 编译零警告；公开 API 表面无 `unsafe`（内部可用）。
- 公开 API 全部是安全 `fn`（无 `unsafe fn`）。
- 真 UB（悬垂、双释放、数据竞争），不靠 panic 模拟崩溃；测试用守卫观察逻辑。

## 架构

`src/lib.rs` 重新导出各模块，模块拆分便于独立理解与测试。

```
src/lib.rs          // 公开 API 汇总 + crate 级 #![allow(...)]（保 0 警告）
src/string.rs       // CppString
src/vec.rs          // CppVec + CppVecIter
src/boxed.rs        // CppBox
src/rc.rs           // CppRc
src/thread.rs       // spawn_cpp_dangerous
src/bomb.rs         // DropBomb
src/macros.rs       // cpp_vec!
examples/*.rs       // 演示：uaf、iter_invalidate、double_free、data_race、thread_dangle、bomb
```

## 陷阱清单与实现

| 类型 | 伪装 | 陷阱实现 |
|---|---|---|
| `CppString` | 像 `String` | 包 `Vec<u8>`；`as_str(&self) -> &'static str` 用 `transmute` 伪造 `'static`，实际指向内部缓冲；`Deref<Target=str>`、`From<&str>`、`From<String>`、`len`、`push_str`、`as_bytes`、`Display`、`Clone`（深拷贝） |
| `CppVec<T>` | 像 `Vec` | 包 `Vec<T>`；`iter()` 返回持裸指针、无生命周期绑定的 `CppVecIter`；实现 `IntoIterator for &CppVec`；迭代期间 `push` 允许 → 重分配使迭代器悬垂；`push`、`len`、`Index`、`From<Vec<T>>` |
| `CppBox<T>` | 像 `Box` | 包 `*mut T`；`new`、`From<T>` 分配；`Clone` 复制裸指针不计数；`Drop` 中 `Box::from_raw` 释放 → 双克隆双释放；`Deref`、`DerefMut` |
| `CppRc<T>` | 像 `Rc` | 包 `*mut Inner<T> { value: T, count: Cell<usize> }`；`Clone` 用 `Cell`（非原子）自增、`Drop` 自减归零释放 → 多线程竞争计数错乱；`Deref`、`strong_count` |
| `spawn_cpp_dangerous` | 像 `thread::spawn` | `F: FnOnce()`（无 `'static` 约束）；先 unsize 为 `Box<dyn FnOnce() + '_>` 再 `transmute` 成 `Box<dyn FnOnce() + 'static>`，`thread::spawn` → 闭包捕获的局部变量出作用域后线程仍访问 |
| `cpp_vec!` | 像 `vec!` | 宏展开为 `CppVec::new()` + 若干 `push` |
| `DropBomb` | 普通结构体 | `new()` 创建 CppString 并保存其 `as_str()`（`'static`）；`Drop` 中先 `drop_in_place` 掉 CppString（释放缓冲）再读该 `&str` → 读已释放内存 |

### 关键实现细节

- **CppString `as_str`**：`unsafe { transmute::<&str, &'static str>(from_utf8_unchecked(&self.0)) }`
- **spawn 闭包升 `'static`**：
  ```rust
  let boxed: Box<dyn FnOnce() + '_> = Box::new(f);          // 合法，生命周期为局部作用域
  let static_boxed: Box<dyn FnOnce() + 'static> = unsafe { transmute(boxed) };
  thread::spawn(move || static_boxed())
  ```
- **CppVecIter**：持有 `*const T`（头/尾），无 `PhantomData<&T>` 借用绑定，故 `push` 与迭代并发合法化（编译层陷阱）。
- **DropBomb Drop**：`unsafe { ptr::drop_in_place(&mut self.s) }` 后 `self.dangling.len()`。

## 测试策略

真 UB 无法可靠断言，单元测试断言可确定的部分，examples 演示崩溃：

- **CppBox 双释放**（确定性）：测试内 `PanicOnDoubleDrop` 守卫（thread_local 标记），第二次 drop 时 panic
  → `#[should_panic(expected = "double free")]`。
- **CppRc 计数逻辑**（确定性）：单线程 `clone`/`drop` 后 `strong_count` 断言。
- **cpp_vec! 宏**：展开结果元素正确。
- **CppString**：`as_str()` 返回 `'static`、`Deref` 行为正常（不触发 UAF 的路径）。
- **CppVec 迭代器**：`for x in &v` 内调用 `push` 能编译通过（陷阱成立）。
- **examples**（崩溃演示，`cargo run --example <name>`）：
  `uaf`（持 `'static` str 后 drop）、`iter_invalidate`（迭代中 push）、`double_free`（双 clone 后 drop）、
  `data_race`（多线程 clone/drop CppRc）、`thread_dangle`（spawn 捕获局部变量）、`bomb`（DropBomb）。

## 风险备注

- 真 UB 在 `cargo test` 中可能偶发段错误（取决于分配器回收行为）；守卫保证逻辑可观察，不做崩溃断言。
- 数据竞争无法在单线程测试断言，交给 `data_race` example 演示 + `strong_count` 逻辑单测兜底。
- 若测试因真 UB 不稳定，回退策略：双释放守卫测试改用 `#[should_panic]` 且守卫 panic 优先于分配器双释检查。

## 文档

- README 一句话：`rustc —— 让你用 Rust 的语法，体验 C++ 的崩溃。编译全部通过，运行时看缘分。`
