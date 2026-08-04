//! Vektra 第一阶段 Theme Profile 校验。

use crate::{dtcg::ResolvedTokens, error::ThemeError};

const SEMANTIC_TOKENS: &[&str] = &[
    "semantic.background",
    "semantic.surface",
    "semantic.foreground",
    "semantic.primary",
    "semantic.on-primary",
    "semantic.secondary",
    "semantic.on-secondary",
    "semantic.accent",
    "semantic.on-accent",
    "semantic.destructive",
    "semantic.on-destructive",
    "semantic.muted",
    "semantic.on-muted",
    "semantic.border",
    "semantic.input-border",
    "semantic.ring",
    "semantic.disabled-background",
    "semantic.disabled-foreground",
    "semantic.disabled-border",
];

const BUTTON_VARIANTS: &[&str] = &[
    "primary",
    "outline",
    "ghost",
    "destructive",
    "secondary",
    "link",
];

const BUTTON_STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];
const BUTTON_SIZES: &[&str] = &["xs", "sm", "md", "lg"];
const CHECKBOX_VISUAL_STATES: &[&str] = &["unchecked", "checked", "indeterminate"];
const CHECKBOX_STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];
const CHECKBOX_SIZES: &[&str] = &["xs", "sm", "md", "lg"];

/// 校验解析后的 token 是否满足 Vektra 第一阶段组件需求。
pub fn validate(tokens: &ResolvedTokens) -> Result<(), ThemeError> {
    for path in SEMANTIC_TOKENS {
        tokens.required(path)?;
    }
    tokens.required("icon.size.default")?;
    tokens.required("button.border-width")?;
    tokens.required("button.focus-width")?;

    // 旧版完整 Tooltip 扩展只包含这些内容 token。新增定位和阴影 token 均有
    // foundation/semantic fallback，不能把旧主题变成不合法主题。
    let tooltip_fields = [
        "background",
        "foreground",
        "border",
        "border-width",
        "padding-x",
        "padding-y",
        "radius",
        "font-size",
        "line-height",
        "max-width",
    ];
    let has_tooltip_extension = tooltip_fields
        .iter()
        .any(|field| tokens.get(&format!("tooltip.{field}")).is_some());
    if has_tooltip_extension {
        for field in tooltip_fields {
            tokens.required(&format!("tooltip.{field}"))?;
        }
    }

    for variant in BUTTON_VARIANTS {
        for state in BUTTON_STATES {
            for field in ["background", "foreground", "border"] {
                tokens.required(&format!("button.variant.{variant}.{state}.{field}"))?;
            }
        }

        let selected_prefix = format!("button.variant.{variant}.selected");
        let has_selected_extension = BUTTON_STATES.iter().any(|state| {
            ["background", "foreground", "border"].iter().any(|field| {
                tokens
                    .get(&format!("{selected_prefix}.{state}.{field}"))
                    .is_some()
            })
        });
        if has_selected_extension {
            for state in BUTTON_STATES {
                for field in ["background", "foreground", "border"] {
                    tokens.required(&format!("{selected_prefix}.{state}.{field}"))?;
                }
            }
        }
    }

    for size in BUTTON_SIZES {
        for field in [
            "height",
            "padding-x",
            "font-size",
            "radius",
            "icon-size",
            "content-gap",
        ] {
            tokens.required(&format!("button.size.{size}.{field}"))?;
        }
    }

    let checkbox_state_fields = ["background", "box-background", "border", "icon", "label"];
    let has_checkbox_state_extension = CHECKBOX_VISUAL_STATES.iter().any(|visual_state| {
        CHECKBOX_STATES.iter().any(|state| {
            checkbox_state_fields.iter().any(|field| {
                tokens
                    .get(&format!("checkbox.state.{visual_state}.{state}.{field}"))
                    .is_some()
            })
        })
    });
    if has_checkbox_state_extension {
        tokens.required("checkbox.border-width")?;
        tokens.required("checkbox.focus-width")?;
        for visual_state in CHECKBOX_VISUAL_STATES {
            for state in CHECKBOX_STATES {
                for field in checkbox_state_fields {
                    tokens.required(&format!("checkbox.state.{visual_state}.{state}.{field}"))?;
                }
            }
        }
    }

    let checkbox_size_fields = [
        "box-size",
        "icon-size",
        "label-gap",
        "font-size",
        "line-height",
        "radius",
        "hit-size",
        "hit-padding-x",
        "hit-padding-y",
    ];
    let has_checkbox_size_extension = CHECKBOX_SIZES.iter().any(|size| {
        checkbox_size_fields.iter().any(|field| {
            tokens
                .get(&format!("checkbox.size.{size}.{field}"))
                .is_some()
        })
    });
    if has_checkbox_size_extension {
        for size in CHECKBOX_SIZES {
            for field in checkbox_size_fields {
                tokens.required(&format!("checkbox.size.{size}.{field}"))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_theme;

    #[test]
    fn default_light_passes_profile() {
        let tokens = default_theme::default_tokens(crate::ResolvedThemeMode::Light).unwrap();
        validate(&tokens).unwrap();
    }

    #[test]
    fn default_dark_passes_profile() {
        let tokens = default_theme::default_tokens(crate::ResolvedThemeMode::Dark).unwrap();
        validate(&tokens).unwrap();
    }
}
