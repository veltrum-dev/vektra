# Vektra Icons

Vektra Icons 存放 Vektra 自带组件会直接使用的少量内置 SVG 图标。它们用于按钮、
图标按钮、搜索操作、设置入口等框架级界面语义，不作为通用图标库发布。应用自己的图标可以
通过自定义 `AssetSource` 和 `IconSource::asset(...)` 提供。

## 图标规格

- SVG 必须使用 `viewBox="0 0 16 16"`。
- outlined 图标使用 `1.2px` 描边，并保留 `stroke="currentColor"`。
- filled 图标使用填充形状表达主体，并保留 `fill="currentColor"`。
- 图标主体应尽量落在内部 `12x12` 光学边界内，也就是视觉重心围绕画布中心。
- 避免 clipping mask、复杂 group、内联样式和无用 metadata。导出后建议用
  SVGOMG 简化层级和属性。

## 命名

- SVG 文件放在 `assets/icons/*.svg`，文件名使用 `snake_case`。
- `IconName` 变体放在 `crates/icons/src/icons.rs`，变体名使用 `PascalCase`。
- 通用含义可使用简短名称，例如 `settings.svg` / `IconName::Settings`。
- 强上下文图标应使用功能前缀，例如 `tool_web.svg` / `IconName::ToolWeb`、
  `repl_play.svg` / `IconName::ReplPlay`、`debug_step_into.svg` /
  `IconName::DebugStepInto`，避免未来同名图标含义漂移。

## 来源

图标可以来自 Lucide、Phosphor，或由 Vektra 自行设计。无论来源如何，都要确认
许可证允许使用，并在 `SOURCE.md` 或对应许可证文件中记录来源、版本、原始路径和
Vektra 调整说明。来自第三方的 SVG 如果经过 16x16 网格、描边或路径调整，来源说明
必须写明它不是未经修改的上游原文件。

## 新增图标流程

1. 把 SVG 添加到 `assets/icons/*.svg`。
2. 在 `crates/icons/src/icons.rs` 为它新增 `IconName` 变体和 `path()` 映射。
3. 更新 `IconName::ALL`。
4. 更新许可证或来源记录。
5. 由 Vektra 维护者检查视觉一致性。
6. 运行图标资源一致性测试：

```bash
cargo test -p vektra-icons
```

如果图标用于 Vektra 内置组件，还需要在启用 `icons` feature 的调用处使用
`IconName::YourIcon`；应用级图标继续使用 `IconSource::asset("icons/custom.svg")` 或
自己的 `IntoIconSource` 实现。
