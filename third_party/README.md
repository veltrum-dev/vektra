# 第三方兼容补丁

这里固定了两个尚未发布 Rust future-incompatibility 修复的间接依赖：

- `block 0.1.6`：把用作外部静态符号类型的不可实例化枚举改为 C 布局的新类型，避免 `uninhabited_static`；同时为 FFI 声明补充明确的 C ABI，避免 `missing_abi`。
- `proc-macro-error2 2.0.1`：公开其宏内部需要重新导出的 `proc_macro` crate，避免 `pub_use_of_private_extern_crate`。

除上述兼容修改外，源码来自 crates.io 对应版本。根 `Cargo.toml` 的
`[patch.crates-io]` 在上游发布修复版本前使用这里的源码。
