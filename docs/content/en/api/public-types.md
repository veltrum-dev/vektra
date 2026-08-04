# Vektra Public Type Index

## Components and Capabilities

| Root export | Purpose |
| --- | --- |
| `Button`, `ButtonVariant` | [Button component](/en/components/button) |
| `Checkbox` | [Checkbox component](/en/components/checkbox) |
| `Radio`, `RadioGroup` | [Radio components](/en/components/radio) |
| `Switch` | [Switch component](/en/components/switch) |
| `SwitchContent` | [Switch track state content](/en/components/switch#api) |
| `IconButton`, `IconButtonVariant` | [IconButton component](/en/components/icon-button) |
| `Icon`, `IconSource`, `IntoIconSource`, `IconName` | Icons and asset sources; the same-name `IntoIconSource` exports include the trait and derive macro, while `IconName` requires the `icons` feature |
| `Tooltip`, `TooltipPlacement` | [Tooltip component](/en/components/tooltip) |
| `Changeable`, `Clickable`, `Focusable`, `Disableable`, `Sizable` | [Capability traits](/en/api/) |

## Theme, Size, and Assets

| Root export | Purpose |
| --- | --- |
| `ComponentSize`, `component_size`, `set_component_size` | Shared semantic sizes and the global default |
| `ThemeMode`, `ResolvedThemeMode` | Requested and resolved theme modes |
| `ResolvedTheme`, `SemanticColors`, `RadioTokens`, `RadioStateTokens`, `RadioSizeTokens`, `TooltipTokens` | Public resolved-theme types |
| `theme_mode`, `set_theme_mode`, `resolved_theme_mode` | Read, set, and resolve theme mode |
| `current_theme`, `semantic_colors` | Current window theme and semantic colors |
| `assets::Assets`, `assets::AssetsWithOverrides` | GPUI asset source and override composition |

Use <a href="../../api/rust/vektra/">rustdoc</a> for complete methods, variants, feature gates, and trait implementations. The root facade is the primary application API; internal workspace crates matter only for explicitly re-exported or advanced direct-use scenarios.
