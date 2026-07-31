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

/// 校验解析后的 token 是否满足 Vektra 第一阶段组件需求。
pub fn validate(tokens: &ResolvedTokens) -> Result<(), ThemeError> {
    for path in SEMANTIC_TOKENS {
        tokens.required(path)?;
    }
    tokens.required("icon.size.default")?;
    tokens.required("button.border-width")?;
    tokens.required("button.focus-width")?;

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
