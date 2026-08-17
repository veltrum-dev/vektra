# Vektra 公共类型索引

## 组件与能力

| 根导出 | 说明 |
| --- | --- |
| `Button`, `ButtonVariant` | [Button 组件](/components/button) |
| `Checkbox` | [Checkbox 组件](/components/checkbox) |
| `Radio`, `RadioGroup` | [Radio 单选组件](/components/radio) |
| `Switch` | [Switch 组件](/components/switch) |
| `SwitchContent` | [Switch 轨道状态内容](/components/switch#api) |
| `Input`, `InputState`, `InputVariant`, `InputClear`, `InputEvent` | [Input 组件](/components/input) |
| `IconButton`, `IconButtonVariant` | [IconButton 组件](/components/icon-button) |
| `Icon`, `IconSource`, `IntoIconSource`, `IconName` | 图标与资源来源；`IntoIconSource` 同名导出包含 trait 与 derive macro，`IconName` 需要 `icons` feature |
| `Tooltip`, `TooltipPlacement` | [Tooltip 组件](/components/tooltip) |
| `ScrollArea`, `ScrollAxis`, `ScrollVisibility`, `ScrollGutter`, `ScrollbarConfig`, `ScrollableExt` | [Scrollbar 组件](/components/scrollbar) |
| `Changeable`, `Clickable`, `Focusable`, `Disableable`, `Sizable` | [能力 traits](/api/) |

## 主题、尺寸与资源

| 根导出 | 说明 |
| --- | --- |
| `ComponentSize`, `component_size`, `set_component_size` | 共享语义尺寸与全局默认值 |
| `ThemeMode`, `ResolvedThemeMode` | 主题选择与解析结果 |
| `ResolvedTheme`, `SemanticColors`, `InputTokens`, `InputStateTokens`, `InputSizeTokens`, `InputVariantKind`, `InputVisualState`, `SelectTokens`, `SelectTriggerStateTokens`, `SelectOptionStateTokens`, `SelectSizeTokens`, `SelectTriggerState`, `SelectOptionState`, `ThemeSize`, `RadioTokens`, `RadioStateTokens`, `RadioSizeTokens`, `TooltipTokens`, `ScrollbarTokens` | 解析后的主题公共类型；Input/Select 使用构造期严格验证后的强类型索引 |
| `theme_mode`, `set_theme_mode`, `resolved_theme_mode` | 读取、设置并解析主题模式 |
| `current_theme`, `semantic_colors` | 当前窗口主题与语义颜色 |
| `assets::Assets`, `assets::AssetsWithOverrides` | 传给 GPUI `with_assets` 的资源源及覆盖组合 |

完整方法、枚举变体、feature 条件和 trait 实现以 <a href="./rust/vektra/">rustdoc</a> 为准。根 facade 是应用首选 API；`vektra-assets`、`vektra-theme`、`vektra-icons`、`vektra-macros` 的类型只在根 crate 明确 re-export 或高级直接使用场景下出现。
