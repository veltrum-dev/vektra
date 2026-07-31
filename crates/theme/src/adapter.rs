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

/// Icon 组件所需的 GPUI token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconTokens {
    /// 未显式指定时的正方形尺寸。
    pub default_size: Pixels,
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
        Ok(ButtonStateTokens {
            background: color(&self.tokens, &format!("{prefix}.background"))?,
            foreground: color(&self.tokens, &format!("{prefix}.foreground"))?,
            border: color(&self.tokens, &format!("{prefix}.border"))?,
        })
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
}

fn color(tokens: &ResolvedTokens, path: &str) -> Result<Hsla, ThemeError> {
    Ok(tokens.color(path)?.to_hsla())
}

fn dimension(tokens: &ResolvedTokens, path: &str) -> Result<Pixels, ThemeError> {
    Ok(tokens.dimension(path)?.to_pixels())
}
