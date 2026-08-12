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
const INPUT_VARIANTS: &[&str] = &["outline", "filled", "borderless", "underline"];
const INPUT_STATES: &[&str] = &[
    "normal",
    "hover",
    "focus-visible",
    "invalid",
    "invalid-focus-visible",
    "read-only",
    "disabled",
];
const INPUT_SIZES: &[&str] = &["xs", "sm", "md", "lg"];
const CHECKBOX_VISUAL_STATES: &[&str] = &["unchecked", "checked", "indeterminate"];
const CHECKBOX_STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];
const CHECKBOX_SIZES: &[&str] = &["xs", "sm", "md", "lg"];
const RADIO_SELECTION_STATES: &[&str] = &["unselected", "selected"];
const RADIO_STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];
const RADIO_SIZES: &[&str] = &["xs", "sm", "md", "lg"];
const SELECT_TRIGGER_STATES: &[&str] = &[
    "normal",
    "hover",
    "pressed",
    "focus-visible",
    "open",
    "disabled",
];
const SELECT_OPTION_STATES: &[&str] = &["normal", "hover", "active", "selected", "disabled"];
const SELECT_SIZES: &[&str] = &["xs", "sm", "md", "lg"];
const SWITCH_VISUAL_STATES: &[&str] = &["unchecked", "checked"];
const SWITCH_STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];
const SWITCH_SIZES: &[&str] = &["xs", "sm", "md", "lg"];

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

    let scrollbar_fields = [
        "track",
        "thumb",
        "thumb-hover",
        "thumb-pressed",
        "focus-ring",
        "thickness",
        "thumb-hover-thickness",
        "hit-thickness",
        "min-thumb-length",
        "radius",
        "focus-width",
    ];
    let has_scrollbar_extension = scrollbar_fields
        .iter()
        .any(|field| tokens.get(&format!("scrollbar.{field}")).is_some());
    if has_scrollbar_extension {
        for field in scrollbar_fields {
            tokens.required(&format!("scrollbar.{field}"))?;
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

    let input_state_fields = [
        "background",
        "foreground",
        "placeholder",
        "border",
        "caret",
        "selection",
        "affix",
        "status",
    ];
    let has_input_state_extension = INPUT_VARIANTS.iter().any(|variant| {
        INPUT_STATES.iter().any(|state| {
            input_state_fields.iter().any(|field| {
                tokens
                    .get(&format!("input.variant.{variant}.{state}.{field}"))
                    .is_some()
            })
        })
    });
    if has_input_state_extension {
        tokens.required("input.border-width")?;
        tokens.required("input.focus-width")?;
        tokens.required("input.caret-width")?;
        for variant in INPUT_VARIANTS {
            for state in INPUT_STATES {
                for field in input_state_fields {
                    tokens.required(&format!("input.variant.{variant}.{state}.{field}"))?;
                }
            }
        }
    }

    let input_size_fields = [
        "height",
        "padding-x",
        "font-size",
        "line-height",
        "radius",
        "slot-size",
        "icon-size",
        "status-size",
        "gap",
    ];
    let has_input_size_extension = INPUT_SIZES.iter().any(|size| {
        input_size_fields
            .iter()
            .any(|field| tokens.get(&format!("input.size.{size}.{field}")).is_some())
    });
    if has_input_size_extension {
        for size in INPUT_SIZES {
            for field in input_size_fields {
                tokens.required(&format!("input.size.{size}.{field}"))?;
            }
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

    let radio_state_fields = [
        "background",
        "indicator-background",
        "border",
        "dot",
        "label",
        "description",
    ];
    let has_radio_state_extension = RADIO_SELECTION_STATES.iter().any(|selection| {
        RADIO_STATES.iter().any(|state| {
            radio_state_fields.iter().any(|field| {
                tokens
                    .get(&format!("radio.state.{selection}.{state}.{field}"))
                    .is_some()
            })
        })
    });
    if has_radio_state_extension {
        tokens.required("radio.border-width")?;
        tokens.required("radio.focus-width")?;
        for selection in RADIO_SELECTION_STATES {
            for state in RADIO_STATES {
                for field in radio_state_fields {
                    tokens.required(&format!("radio.state.{selection}.{state}.{field}"))?;
                }
            }
        }
    }

    let radio_size_fields = [
        "indicator-size",
        "dot-size",
        "label-gap",
        "description-gap",
        "font-size",
        "line-height",
        "description-font-size",
        "description-line-height",
        "hit-size",
        "hit-padding-x",
        "hit-padding-y",
        "group-gap",
    ];
    let has_radio_size_extension = RADIO_SIZES.iter().any(|size| {
        radio_size_fields
            .iter()
            .any(|field| tokens.get(&format!("radio.size.{size}.{field}")).is_some())
    });
    if has_radio_size_extension {
        for size in RADIO_SIZES {
            for field in radio_size_fields {
                tokens.required(&format!("radio.size.{size}.{field}"))?;
            }
        }
    }

    let select_common_fields = ["border-width", "focus-width", "group-label"];
    let select_popup_fields = [
        "background",
        "border",
        "border-width",
        "radius",
        "padding",
        "shadow-color",
        "shadow-offset-y",
        "shadow-blur",
        "shadow-spread",
        "anchor-gap",
        "viewport-padding",
        "max-height",
    ];
    let select_status_fields = ["loading", "empty", "error"];
    let select_trigger_fields = [
        "background",
        "foreground",
        "placeholder",
        "border",
        "indicator",
    ];
    let select_option_fields = ["background", "foreground", "description", "indicator"];
    let select_size_fields = [
        "height",
        "padding-x",
        "font-size",
        "line-height",
        "radius",
        "icon-size",
        "indicator-size",
        "content-gap",
        "option-padding-x",
        "option-padding-y",
        "description-font-size",
        "description-line-height",
        "group-padding-y",
    ];
    let has_select_extension = select_common_fields
        .iter()
        .any(|field| tokens.get(&format!("select.{field}")).is_some())
        || select_popup_fields
            .iter()
            .any(|field| tokens.get(&format!("select.popup.{field}")).is_some())
        || SELECT_TRIGGER_STATES.iter().any(|state| {
            select_trigger_fields.iter().any(|field| {
                tokens
                    .get(&format!("select.trigger.{state}.{field}"))
                    .is_some()
            })
        });
    if has_select_extension {
        for field in select_common_fields {
            tokens.required(&format!("select.{field}"))?;
        }
        for field in select_popup_fields {
            tokens.required(&format!("select.popup.{field}"))?;
        }
        for field in select_status_fields {
            tokens.required(&format!("select.status.{field}"))?;
        }
        for state in SELECT_TRIGGER_STATES {
            for field in select_trigger_fields {
                tokens.required(&format!("select.trigger.{state}.{field}"))?;
            }
        }
        for state in SELECT_OPTION_STATES {
            for field in select_option_fields {
                tokens.required(&format!("select.option.{state}.{field}"))?;
            }
        }
        for size in SELECT_SIZES {
            for field in select_size_fields {
                tokens.required(&format!("select.size.{size}.{field}"))?;
            }
        }
    }

    let switch_state_fields = ["track-background", "track-border", "thumb", "label"];
    let has_switch_state_extension = SWITCH_VISUAL_STATES.iter().any(|visual_state| {
        SWITCH_STATES.iter().any(|state| {
            switch_state_fields.iter().any(|field| {
                tokens
                    .get(&format!("switch.state.{visual_state}.{state}.{field}"))
                    .is_some()
            })
        })
    });
    if has_switch_state_extension {
        tokens.required("switch.border-width")?;
        tokens.required("switch.focus-width")?;
        for visual_state in SWITCH_VISUAL_STATES {
            for state in SWITCH_STATES {
                for field in switch_state_fields {
                    tokens.required(&format!("switch.state.{visual_state}.{state}.{field}"))?;
                }
            }
        }
    }

    let has_switch_content_state_extension = SWITCH_VISUAL_STATES.iter().any(|visual_state| {
        SWITCH_STATES.iter().any(|state| {
            tokens
                .get(&format!("switch.state.{visual_state}.{state}.content"))
                .is_some()
        })
    });
    if has_switch_content_state_extension {
        for visual_state in SWITCH_VISUAL_STATES {
            for state in SWITCH_STATES {
                tokens.required(&format!("switch.state.{visual_state}.{state}.content"))?;
            }
        }
    }

    let has_switch_spinner_state_extension = SWITCH_VISUAL_STATES.iter().any(|visual_state| {
        SWITCH_STATES.iter().any(|state| {
            tokens
                .get(&format!("switch.state.{visual_state}.{state}.spinner"))
                .is_some()
        })
    });
    if has_switch_spinner_state_extension {
        for visual_state in SWITCH_VISUAL_STATES {
            for state in SWITCH_STATES {
                tokens.required(&format!("switch.state.{visual_state}.{state}.spinner"))?;
            }
        }
    }

    let switch_size_fields = [
        "track-width",
        "track-height",
        "track-padding",
        "thumb-size",
        "track-radius",
        "thumb-radius",
        "label-gap",
        "font-size",
        "line-height",
        "hit-size",
        "hit-padding-x",
        "hit-padding-y",
    ];
    let has_switch_size_extension = SWITCH_SIZES.iter().any(|size| {
        switch_size_fields
            .iter()
            .any(|field| tokens.get(&format!("switch.size.{size}.{field}")).is_some())
    });
    if has_switch_size_extension {
        for size in SWITCH_SIZES {
            for field in switch_size_fields {
                tokens.required(&format!("switch.size.{size}.{field}"))?;
            }
        }
    }

    let switch_content_size_fields = [
        "content-track-height",
        "content-track-padding",
        "content-thumb-size",
        "content-slot-gap",
        "content-edge-padding",
        "content-icon-size",
        "content-gap",
        "content-max-text-width",
    ];
    let has_switch_content_size_extension = SWITCH_SIZES.iter().any(|size| {
        switch_content_size_fields
            .iter()
            .any(|field| tokens.get(&format!("switch.size.{size}.{field}")).is_some())
    });
    if has_switch_content_size_extension {
        for size in SWITCH_SIZES {
            for field in switch_content_size_fields {
                tokens.required(&format!("switch.size.{size}.{field}"))?;
            }
        }
    }

    let has_switch_spinner_size_extension = SWITCH_SIZES.iter().any(|size| {
        tokens
            .get(&format!("switch.size.{size}.spinner-size"))
            .is_some()
    });
    if has_switch_spinner_size_extension {
        for size in SWITCH_SIZES {
            tokens.required(&format!("switch.size.{size}.spinner-size"))?;
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
