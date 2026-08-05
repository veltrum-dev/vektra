# API 参考

这里记录 `vektra` 根 crate 面向应用的公共表面。概念、行为约束和可编译示例放在 VitePress；穷尽式符号、实现和自动生成的关联项放在 <a href="./rust/vektra/">完整 Rust API（rustdoc）</a>。

## API 来源

- `vektra::Button`、`Checkbox`、`Radio`、`RadioGroup`、`Switch`、`Input`、`IconButton`、`Tooltip`、能力 traits、主题和共享类型由 Vektra 定义，本站负责解释并提供完整 rustdoc。
- `gpui::ClickEvent`、`Window`、`App`、`Context<T>` 等由 GPUI 定义。它们出现在 Vektra 回调签名中，但 Vektra 不复制维护其完整定义；请从 [GPUI 依赖类型](./gpui-types) 跳转到锁定源码。
- GPUI 与 `gpui_platform` 固定在 commit `82aef44308540b576e4e51fb379efa71614e5c91`。源码链接不指向会漂移的 `main`。

## 导航

- 能力 traits：[Changeable](./changeable)、[Clickable](./clickable)、[Focusable](./focusable)、[Disableable](./disableable)、[Sizable](./sizable)
- [回调模型与 `_in` 约定](./callbacks)
- [Vektra 公共类型索引](./public-types)
- [GPUI 依赖类型索引](./gpui-types)

以根 `vektra` facade 为首选导入入口；只有资源组合等明确场景才需要直接理解内部 workspace crate。
