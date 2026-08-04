//! 将解析后的 DTCG token 转换为 GPUI 类型。

use crate::{dtcg::ResolvedTokens, error::ThemeError, mode::ResolvedThemeMode};
use gpui::{Hsla, Pixels};

/// GPUI 可直接使用的语义颜色。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticColors {
    /// 应用背景。
    pub background: Hsla,
    /// 表面背景。
    pub surface: Hsla,
    /// 默认前景色。
    pub foreground: Hsla,
    /// 主操作颜色。
    pub primary: Hsla,
    /// 主操作前景色。
    pub on_primary: Hsla,
    /// 次要背景。
    pub secondary: Hsla,
    /// 次要前景色。
    pub on_secondary: Hsla,
    /// 轻强调背景。
    pub accent: Hsla,
    /// 轻强调前景色。
    pub on_accent: Hsla,
    /// 危险操作颜色。
    pub destructive: Hsla,
    /// 危险操作前景色。
    pub on_destructive: Hsla,
    /// 弱背景。
    pub muted: Hsla,
    /// 弱前景色。
    pub on_muted: Hsla,
    /// 普通边框。
    pub border: Hsla,
    /// 输入/控件边框。
    pub input_border: Hsla,
    /// Focus ring 颜色。
    pub ring: Hsla,
    /// disabled 背景。
    pub disabled_background: Hsla,
    /// disabled 前景色。
    pub disabled_foreground: Hsla,
    /// disabled 边框。
    pub disabled_border: Hsla,
}

/// Button 某个 variant/state 的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonStateTokens {
    /// 背景色。
    pub background: Hsla,
    /// 文字色。
    pub foreground: Hsla,
    /// 边框色。
    pub border: Hsla,
}

/// Button 某个 size 的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonSizeTokens {
    /// 高度。
    pub height: Pixels,
    /// 水平内边距。
    pub padding_x: Pixels,
    /// 字号。
    pub font_size: Pixels,
    /// 圆角。
    pub radius: Pixels,
    /// Button 内图标的正方形尺寸。
    pub icon_size: Pixels,
    /// Button 内容项之间的间距。
    pub content_gap: Pixels,
}

/// Button 组件所需的全部 GPUI token。
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonTokens {
    /// 边框宽度。
    pub border_width: Pixels,
    /// focus-visible 边框宽度。
    pub focus_width: Pixels,
}

/// Checkbox 某个 state 的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxStateTokens {
    /// 交互根背景色。
    pub background: Hsla,
    /// 方框背景色。
    pub box_background: Hsla,
    /// 方框边框色。
    pub border: Hsla,
    /// 状态图标色。
    pub icon: Hsla,
    /// label 文本色。
    pub label: Hsla,
}

/// Checkbox 某个 size 的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxSizeTokens {
    /// 方框正方形尺寸。
    pub box_size: Pixels,
    /// 状态图标正方形尺寸。
    pub icon_size: Pixels,
    /// label 与方框之间的间距。
    pub label_gap: Pixels,
    /// 字号。
    pub font_size: Pixels,
    /// 行高。
    pub line_height: Pixels,
    /// 圆角。
    pub radius: Pixels,
    /// 点击目标最小尺寸。
    pub hit_size: Pixels,
    /// 点击目标水平内边距。
    pub hit_padding_x: Pixels,
    /// 点击目标垂直内边距。
    pub hit_padding_y: Pixels,
}

/// Checkbox 组件所需的公共 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxTokens {
    /// 方框边框宽度。
    pub border_width: Pixels,
    /// focus-visible 边框宽度。
    pub focus_width: Pixels,
}

/// Icon 组件所需的 GPUI token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconTokens {
    /// 未显式指定时的正方形尺寸。
    pub default_size: Pixels,
}

/// Tooltip 组件所需的 GPUI token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipTokens {
    /// 背景色。
    pub background: Hsla,
    /// 文字色。
    pub foreground: Hsla,
    /// 边框色。
    pub border: Hsla,
    /// 边框宽度。
    pub border_width: Pixels,
    /// 水平内边距。
    pub padding_x: Pixels,
    /// 垂直内边距。
    pub padding_y: Pixels,
    /// 圆角。
    pub radius: Pixels,
    /// 字号。
    pub font_size: Pixels,
    /// 行高。
    pub line_height: Pixels,
    /// 最大宽度。
    pub max_width: Pixels,
    /// Tooltip 与 trigger 之间的最小间距。
    pub anchor_gap: Pixels,
    /// Tooltip、箭头与阴影距离视口边缘的安全内边距。
    pub viewport_padding: Pixels,
    /// 箭头在气泡边缘方向上的宽度。
    pub arrow_width: Pixels,
    /// 箭头从气泡指向 trigger 的高度。
    pub arrow_height: Pixels,
    /// 阴影颜色。
    pub shadow_color: Hsla,
    /// 阴影水平偏移。
    pub shadow_offset_x: Pixels,
    /// 阴影垂直偏移。
    pub shadow_offset_y: Pixels,
    /// 阴影模糊半径。
    pub shadow_blur: Pixels,
    /// 阴影扩展半径。
    pub shadow_spread: Pixels,
}

/// 解析并转换后的 Vektra 主题。
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTheme {
    /// 实际使用的 Light/Dark 模式。
    pub mode: ResolvedThemeMode,
    /// 常用语义颜色。
    pub semantic: SemanticColors,
    /// Icon 公共 token。
    pub icon: IconTokens,
    /// Button 公共 token。
    pub button: ButtonTokens,
    /// Checkbox 公共 token。
    pub checkbox: CheckboxTokens,
    /// Tooltip 公共 token。
    pub tooltip: TooltipTokens,
    tokens: ResolvedTokens,
}

impl ResolvedTheme {
    /// 从已解析 DTCG token 构造 GPUI adapter 主题。
    pub fn from_tokens(
        mode: ResolvedThemeMode,
        tokens: ResolvedTokens,
    ) -> Result<Self, ThemeError> {
        Ok(Self {
            mode,
            semantic: SemanticColors {
                background: color(&tokens, "semantic.background")?,
                surface: color(&tokens, "semantic.surface")?,
                foreground: color(&tokens, "semantic.foreground")?,
                primary: color(&tokens, "semantic.primary")?,
                on_primary: color(&tokens, "semantic.on-primary")?,
                secondary: color(&tokens, "semantic.secondary")?,
                on_secondary: color(&tokens, "semantic.on-secondary")?,
                accent: color(&tokens, "semantic.accent")?,
                on_accent: color(&tokens, "semantic.on-accent")?,
                destructive: color(&tokens, "semantic.destructive")?,
                on_destructive: color(&tokens, "semantic.on-destructive")?,
                muted: color(&tokens, "semantic.muted")?,
                on_muted: color(&tokens, "semantic.on-muted")?,
                border: color(&tokens, "semantic.border")?,
                input_border: color(&tokens, "semantic.input-border")?,
                ring: color(&tokens, "semantic.ring")?,
                disabled_background: color(&tokens, "semantic.disabled-background")?,
                disabled_foreground: color(&tokens, "semantic.disabled-foreground")?,
                disabled_border: color(&tokens, "semantic.disabled-border")?,
            },
            icon: IconTokens {
                default_size: dimension(&tokens, "icon.size.default")?,
            },
            button: ButtonTokens {
                border_width: dimension(&tokens, "button.border-width")?,
                focus_width: dimension(&tokens, "button.focus-width")?,
            },
            checkbox: CheckboxTokens {
                border_width: optional_dimension(
                    &tokens,
                    "checkbox.border-width",
                    "foundation.border.width",
                )?,
                focus_width: optional_dimension(
                    &tokens,
                    "checkbox.focus-width",
                    "foundation.border.focus",
                )?,
            },
            tooltip: TooltipTokens {
                background: optional_color(&tokens, "tooltip.background", "semantic.surface")?,
                foreground: optional_color(&tokens, "tooltip.foreground", "semantic.foreground")?,
                border: optional_color(&tokens, "tooltip.border", "semantic.border")?,
                border_width: optional_dimension(
                    &tokens,
                    "tooltip.border-width",
                    "foundation.border.width",
                )?,
                padding_x: optional_dimension(&tokens, "tooltip.padding-x", "foundation.space.2")?,
                padding_y: optional_dimension(
                    &tokens,
                    "tooltip.padding-y",
                    "foundation.space.1_5",
                )?,
                radius: optional_dimension(&tokens, "tooltip.radius", "foundation.radius.md")?,
                font_size: optional_dimension(
                    &tokens,
                    "tooltip.font-size",
                    "foundation.font.size.sm",
                )?,
                line_height: optional_dimension(
                    &tokens,
                    "tooltip.line-height",
                    "foundation.space.4",
                )?,
                max_width: if tokens.get("tooltip.max-width").is_some() {
                    dimension(&tokens, "tooltip.max-width")?
                } else {
                    gpui::px(280.)
                },
                anchor_gap: optional_dimension(
                    &tokens,
                    "tooltip.anchor-gap",
                    "foundation.space.1",
                )?,
                viewport_padding: optional_dimension(
                    &tokens,
                    "tooltip.viewport-padding",
                    "foundation.space.2",
                )?,
                arrow_width: optional_dimension(
                    &tokens,
                    "tooltip.arrow-width",
                    "foundation.space.3",
                )?,
                arrow_height: optional_dimension(
                    &tokens,
                    "tooltip.arrow-height",
                    "foundation.space.1_5",
                )?,
                shadow_color: if tokens.get("tooltip.shadow-color").is_some() {
                    color(&tokens, "tooltip.shadow-color")?
                } else {
                    color(&tokens, "semantic.foreground")?.opacity(0.16)
                },
                shadow_offset_x: optional_dimension(
                    &tokens,
                    "tooltip.shadow-offset-x",
                    "foundation.space.0",
                )?,
                shadow_offset_y: optional_dimension(
                    &tokens,
                    "tooltip.shadow-offset-y",
                    "foundation.space.1",
                )?,
                shadow_blur: optional_dimension(
                    &tokens,
                    "tooltip.shadow-blur",
                    "foundation.space.2",
                )?,
                shadow_spread: optional_dimension(
                    &tokens,
                    "tooltip.shadow-spread",
                    "foundation.space.0",
                )?,
            },
            tokens,
        })
    }

    /// 读取 Button variant/state 样式 token。
    pub fn button_state(
        &self,
        variant: &str,
        state: &str,
    ) -> Result<ButtonStateTokens, ThemeError> {
        let prefix = format!("button.variant.{variant}.{state}");
        button_state(&self.tokens, &prefix)
    }

    /// 读取 Button 的可选 selected variant/state 样式 token。
    ///
    /// 旧主题完全不提供该扩展时返回 `Ok(None)`，调用方可使用既有
    /// pressed/focus-visible/disabled token 组合回退；若主题开始提供某个 selected
    /// 状态，则该状态的 background、foreground 和 border 必须完整且类型正确。
    pub fn button_selected_state(
        &self,
        variant: &str,
        state: &str,
    ) -> Result<Option<ButtonStateTokens>, ThemeError> {
        let prefix = format!("button.variant.{variant}.selected.{state}");
        let fields = ["background", "foreground", "border"];
        if fields
            .iter()
            .all(|field| self.tokens.get(&format!("{prefix}.{field}")).is_none())
        {
            return Ok(None);
        }

        button_state(&self.tokens, &prefix).map(Some)
    }

    /// 读取 Button size token。
    pub fn button_size(&self, size: &str) -> Result<ButtonSizeTokens, ThemeError> {
        let prefix = format!("button.size.{size}");
        Ok(ButtonSizeTokens {
            height: dimension(&self.tokens, &format!("{prefix}.height"))?,
            padding_x: dimension(&self.tokens, &format!("{prefix}.padding-x"))?,
            font_size: dimension(&self.tokens, &format!("{prefix}.font-size"))?,
            radius: dimension(&self.tokens, &format!("{prefix}.radius"))?,
            icon_size: dimension(&self.tokens, &format!("{prefix}.icon-size"))?,
            content_gap: dimension(&self.tokens, &format!("{prefix}.content-gap"))?,
        })
    }

    /// 读取 Checkbox state token。
    pub fn checkbox_state(
        &self,
        visual_state: &str,
        state: &str,
    ) -> Result<CheckboxStateTokens, ThemeError> {
        let prefix = format!("checkbox.state.{visual_state}.{state}");
        Ok(CheckboxStateTokens {
            background: optional_color(
                &self.tokens,
                &format!("{prefix}.background"),
                "semantic.background",
            )?,
            box_background: optional_color(
                &self.tokens,
                &format!("{prefix}.box-background"),
                checkbox_box_background_fallback(visual_state, state),
            )?,
            border: optional_color(
                &self.tokens,
                &format!("{prefix}.border"),
                checkbox_border_fallback(visual_state, state),
            )?,
            icon: optional_color(
                &self.tokens,
                &format!("{prefix}.icon"),
                checkbox_icon_fallback(visual_state, state),
            )?,
            label: optional_color(
                &self.tokens,
                &format!("{prefix}.label"),
                checkbox_label_fallback(state),
            )?,
        })
    }

    /// 读取 Checkbox size token。
    pub fn checkbox_size(&self, size: &str) -> Result<CheckboxSizeTokens, ThemeError> {
        let prefix = format!("checkbox.size.{size}");
        Ok(CheckboxSizeTokens {
            box_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.box-size"),
                checkbox_box_size_fallback(size),
            )?,
            icon_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.icon-size"),
                checkbox_icon_size_fallback(size),
            )?,
            label_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.label-gap"),
                checkbox_label_gap_fallback(size),
            )?,
            font_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.font-size"),
                checkbox_font_size_fallback(size),
            )?,
            line_height: optional_dimension(
                &self.tokens,
                &format!("{prefix}.line-height"),
                checkbox_line_height_fallback(size),
            )?,
            radius: optional_dimension(
                &self.tokens,
                &format!("{prefix}.radius"),
                "foundation.radius.sm",
            )?,
            hit_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.hit-size"),
                "foundation.space.4",
            )?,
            hit_padding_x: optional_dimension(
                &self.tokens,
                &format!("{prefix}.hit-padding-x"),
                "foundation.space.1",
            )?,
            hit_padding_y: optional_dimension(
                &self.tokens,
                &format!("{prefix}.hit-padding-y"),
                "foundation.space.1",
            )?,
        })
    }
}

fn color(tokens: &ResolvedTokens, path: &str) -> Result<Hsla, ThemeError> {
    Ok(tokens.color(path)?.to_hsla())
}

fn button_state(tokens: &ResolvedTokens, prefix: &str) -> Result<ButtonStateTokens, ThemeError> {
    Ok(ButtonStateTokens {
        background: color(tokens, &format!("{prefix}.background"))?,
        foreground: color(tokens, &format!("{prefix}.foreground"))?,
        border: color(tokens, &format!("{prefix}.border"))?,
    })
}

fn dimension(tokens: &ResolvedTokens, path: &str) -> Result<Pixels, ThemeError> {
    Ok(tokens.dimension(path)?.to_pixels())
}

fn optional_color(tokens: &ResolvedTokens, path: &str, fallback: &str) -> Result<Hsla, ThemeError> {
    color(
        tokens,
        if tokens.get(path).is_some() {
            path
        } else {
            fallback
        },
    )
}

fn optional_dimension(
    tokens: &ResolvedTokens,
    path: &str,
    fallback: &str,
) -> Result<Pixels, ThemeError> {
    dimension(
        tokens,
        if tokens.get(path).is_some() {
            path
        } else {
            fallback
        },
    )
}

fn checkbox_box_background_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-background"
    } else if visual_state == "unchecked" {
        "semantic.background"
    } else {
        "semantic.primary"
    }
}

fn checkbox_border_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "focus-visible" {
        "semantic.ring"
    } else if state == "disabled" {
        "semantic.disabled-border"
    } else if visual_state == "unchecked" {
        "semantic.input-border"
    } else {
        "semantic.primary"
    }
}

fn checkbox_icon_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else if visual_state == "unchecked" {
        "semantic.foreground"
    } else {
        "semantic.on-primary"
    }
}

fn checkbox_label_fallback(state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else {
        "semantic.foreground"
    }
}

fn checkbox_box_size_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.space.3",
        "sm" => "foundation.space.4",
        "md" => "foundation.space.4",
        "lg" => "foundation.space.4",
        _ => "foundation.space.4",
    }
}

fn checkbox_icon_size_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.space.2",
        "sm" => "foundation.space.2_5",
        "md" => "foundation.space.3",
        "lg" => "foundation.space.3",
        _ => "foundation.space.3",
    }
}

fn checkbox_label_gap_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.space.1_5",
        "sm" => "foundation.space.2",
        "md" => "foundation.space.2",
        "lg" => "foundation.space.2_5",
        _ => "foundation.space.2",
    }
}

fn checkbox_font_size_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.font.size.xs",
        "sm" => "foundation.font.size.sm",
        "md" | "lg" => "foundation.font.size.md",
        _ => "foundation.font.size.md",
    }
}

fn checkbox_line_height_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.space.3",
        "sm" => "foundation.space.4",
        "md" | "lg" => "foundation.space.4",
        _ => "foundation.space.4",
    }
}
