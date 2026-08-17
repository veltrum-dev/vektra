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

/// Input 某个 variant/state 的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputStateTokens {
    /// 背景色。
    pub background: Hsla,
    /// 输入文字色。
    pub foreground: Hsla,
    /// placeholder 文字色。
    pub placeholder: Hsla,
    /// 边框或底线颜色。
    pub border: Hsla,
    /// 光标颜色。
    pub caret: Hsla,
    /// 选区背景色。
    pub selection: Hsla,
    /// prefix、suffix 等附属内容的默认前景色。
    pub affix: Hsla,
    /// invalid 状态标记颜色。
    pub status: Hsla,
}

/// Input 某个 size 的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputSizeTokens {
    /// 控件高度。
    pub height: Pixels,
    /// 水平内边距。
    pub padding_x: Pixels,
    /// 输入文字字号。
    pub font_size: Pixels,
    /// 输入文字行高。
    pub line_height: Pixels,
    /// 外壳圆角。
    pub radius: Pixels,
    /// 推荐的紧凑槽位尺寸。
    pub slot_size: Pixels,
    /// 内置清除图标尺寸。
    pub icon_size: Pixels,
    /// invalid 状态图标尺寸。
    pub status_size: Pixels,
    /// 相邻内容间距。
    pub gap: Pixels,
}

/// Input 组件所需的公共 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputTokens {
    /// 普通边框或底线宽度。
    pub border_width: Pixels,
    /// focus-visible 边框或底线宽度。
    pub focus_width: Pixels,
    /// 文本光标宽度。
    pub caret_width: Pixels,
}

/// Input 的公开视觉变体索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum InputVariantKind {
    /// 带完整边框的输入框。
    Outline,
    /// 带填充背景的输入框。
    Filled,
    /// 无常驻边框的输入框。
    Borderless,
    /// 仅显示底线的输入框。
    Underline,
}

/// Input 的公开视觉状态索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum InputVisualState {
    /// 默认状态。
    Normal,
    /// 指针悬停状态。
    Hover,
    /// 键盘焦点可见状态。
    FocusVisible,
    /// 校验失败状态。
    Invalid,
    /// 校验失败且键盘焦点可见。
    InvalidFocusVisible,
    /// 只读状态。
    ReadOnly,
    /// 禁用状态。
    Disabled,
}

/// Input 与 Select 共用的语义尺寸索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum ThemeSize {
    /// 超小尺寸。
    Xs,
    /// 小尺寸。
    Sm,
    /// 中尺寸。
    Md,
    /// 大尺寸。
    Lg,
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

/// Radio 某个选中/交互状态的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioStateTokens {
    /// 单项交互背景色。
    pub background: Hsla,
    /// 圆形指示器背景色。
    pub indicator_background: Hsla,
    /// 圆形指示器边框色。
    pub border: Hsla,
    /// 选中圆点颜色。
    pub dot: Hsla,
    /// 主标签文本色。
    pub label: Hsla,
    /// 描述文本色。
    pub description: Hsla,
}

/// RadioGroup 某个语义尺寸的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioSizeTokens {
    /// 圆形指示器尺寸。
    pub indicator_size: Pixels,
    /// 选中圆点尺寸。
    pub dot_size: Pixels,
    /// 指示器与文本之间的间距。
    pub label_gap: Pixels,
    /// 标签与描述之间的间距。
    pub description_gap: Pixels,
    /// 主标签字号。
    pub font_size: Pixels,
    /// 主标签行高。
    pub line_height: Pixels,
    /// 描述字号。
    pub description_font_size: Pixels,
    /// 描述行高。
    pub description_line_height: Pixels,
    /// 单项最小点击尺寸。
    pub hit_size: Pixels,
    /// 单项水平内边距。
    pub hit_padding_x: Pixels,
    /// 单项垂直内边距。
    pub hit_padding_y: Pixels,
    /// 组内相邻单项间距。
    pub group_gap: Pixels,
}

/// Radio 组件所需的公共 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioTokens {
    /// 指示器边框宽度。
    pub border_width: Pixels,
    /// focus-visible 边框宽度。
    pub focus_width: Pixels,
}

/// Select Trigger 某个交互状态的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectTriggerStateTokens {
    /// Trigger 背景色。
    pub background: Hsla,
    /// 已选内容前景色。
    pub foreground: Hsla,
    /// Placeholder 前景色。
    pub placeholder: Hsla,
    /// Trigger 边框色。
    pub border: Hsla,
    /// 展开指示器颜色。
    pub indicator: Hsla,
}

/// Select Option 某个交互状态的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectOptionStateTokens {
    /// Option 背景色。
    pub background: Hsla,
    /// 主标签前景色。
    pub foreground: Hsla,
    /// 描述前景色。
    pub description: Hsla,
    /// 选中指示器颜色。
    pub indicator: Hsla,
}

/// Select 某个语义尺寸的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectSizeTokens {
    /// Trigger 高度。
    pub height: Pixels,
    /// Trigger 水平内边距。
    pub padding_x: Pixels,
    /// 主文本字号。
    pub font_size: Pixels,
    /// 主文本行高。
    pub line_height: Pixels,
    /// Trigger 圆角。
    pub radius: Pixels,
    /// Option 前置图标尺寸。
    pub icon_size: Pixels,
    /// 展开与选中指示器尺寸。
    pub indicator_size: Pixels,
    /// 相邻内容间距。
    pub content_gap: Pixels,
    /// Option 水平内边距。
    pub option_padding_x: Pixels,
    /// Option 垂直内边距。
    pub option_padding_y: Pixels,
    /// Option 描述字号。
    pub description_font_size: Pixels,
    /// Option 描述行高。
    pub description_line_height: Pixels,
    /// Group label 垂直内边距。
    pub group_padding_y: Pixels,
}

/// Select 组件所需的公共 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectTokens {
    /// Trigger 普通边框宽度。
    pub border_width: Pixels,
    /// Trigger focus-visible 边框宽度。
    pub focus_width: Pixels,
    /// Popup 背景色。
    pub popup_background: Hsla,
    /// Popup 边框色。
    pub popup_border: Hsla,
    /// Popup 边框宽度。
    pub popup_border_width: Pixels,
    /// Popup 圆角。
    pub popup_radius: Pixels,
    /// Popup 内容内边距。
    pub popup_padding: Pixels,
    /// Popup 阴影颜色。
    pub popup_shadow_color: Hsla,
    /// Popup 阴影垂直偏移。
    pub popup_shadow_offset_y: Pixels,
    /// Popup 阴影模糊半径。
    pub popup_shadow_blur: Pixels,
    /// Popup 阴影扩展半径。
    pub popup_shadow_spread: Pixels,
    /// Popup 与 Trigger 的间距。
    pub popup_anchor_gap: Pixels,
    /// Popup 距离视口边缘的安全距离。
    pub popup_viewport_padding: Pixels,
    /// Popup 在空间充足时的最大高度。
    pub popup_max_height: Pixels,
    /// Group label 前景色。
    pub group_label: Hsla,
    /// Loading 状态前景色。
    pub status_loading: Hsla,
    /// Empty 状态前景色。
    pub status_empty: Hsla,
    /// Error 状态前景色。
    pub status_error: Hsla,
}

/// Select Trigger 的公开视觉状态索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum SelectTriggerState {
    /// 默认状态。
    Normal,
    /// 指针悬停状态。
    Hover,
    /// 指针按下状态。
    Pressed,
    /// 键盘焦点可见状态。
    FocusVisible,
    /// 弹层展开状态。
    Open,
    /// 禁用状态。
    Disabled,
}

/// Select Option 的公开视觉状态索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum SelectOptionState {
    /// 默认状态。
    Normal,
    /// 指针悬停状态。
    Hover,
    /// 键盘活动项状态。
    Active,
    /// 已选状态。
    Selected,
    /// 禁用状态。
    Disabled,
}

/// Switch 某个 visual/interaction state 的 GPUI 样式 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchStateTokens {
    /// Track 背景色。
    pub track_background: Hsla,
    /// Track 边框色。
    pub track_border: Hsla,
    /// Thumb 背景色。
    pub thumb: Hsla,
    /// Label 文本色。
    pub label: Hsla,
    /// Track 内状态内容的前景色。
    pub content: Hsla,
    /// Thumb 内 loading spinner 的前景色。
    pub spinner: Hsla,
}

/// Switch 某个 size 的 GPUI 尺寸 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchSizeTokens {
    /// Track 宽度。
    pub track_width: Pixels,
    /// Track 高度。
    pub track_height: Pixels,
    /// Track 内边距。
    pub track_padding: Pixels,
    /// Thumb 正方形尺寸。
    pub thumb_size: Pixels,
    /// Track 圆角。
    pub track_radius: Pixels,
    /// Thumb 圆角。
    pub thumb_radius: Pixels,
    /// Label 与 track 的间距。
    pub label_gap: Pixels,
    /// 字号。
    pub font_size: Pixels,
    /// 行高。
    pub line_height: Pixels,
    /// 点击目标最小尺寸。
    pub hit_size: Pixels,
    /// 点击目标水平内边距。
    pub hit_padding_x: Pixels,
    /// 点击目标垂直内边距。
    pub hit_padding_y: Pixels,
    /// 内容模式的 Track 高度。
    pub content_track_height: Pixels,
    /// 内容模式的 Track 内边距。
    pub content_track_padding: Pixels,
    /// 内容模式的 Thumb 正方形尺寸。
    pub content_thumb_size: Pixels,
    /// Thumb 槽与状态内容槽之间的间距。
    pub content_slot_gap: Pixels,
    /// 状态内容与 Track 逻辑边缘之间的内边距。
    pub content_edge_padding: Pixels,
    /// Track 内状态图标的正方形尺寸。
    pub content_icon_size: Pixels,
    /// 状态图标与文字之间的间距。
    pub content_gap: Pixels,
    /// Track 内状态文字的最大宽度。
    pub content_max_text_width: Pixels,
    /// Thumb 内 loading spinner 的正方形尺寸。
    pub spinner_size: Pixels,
}

/// Switch 组件所需的公共 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchTokens {
    /// Track 边框宽度。
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

/// Scrollbar 组件所需的 GPUI token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarTokens {
    /// 轨道在 hover 或拖动时的背景色。
    pub track: Hsla,
    /// Thumb 默认颜色。
    pub thumb: Hsla,
    /// Thumb hover 颜色。
    pub thumb_hover: Hsla,
    /// Thumb 拖动颜色。
    pub thumb_pressed: Hsla,
    /// 键盘焦点环颜色。
    pub focus_ring: Hsla,
    /// 实际绘制的轨道与 Thumb 宽度。
    pub thickness: Pixels,
    /// Thumb 在 hover 或拖动时的视觉宽度。
    pub thumb_hover_thickness: Pixels,
    /// 鼠标命中区域宽度，也是 `Stable` gutter 的预留宽度。
    pub hit_thickness: Pixels,
    /// Thumb 在主轴上的最小长度。
    pub min_thumb_length: Pixels,
    /// 轨道圆角；Thumb 始终根据自身短边绘制为胶囊形。
    pub radius: Pixels,
    /// 键盘焦点环宽度。
    pub focus_width: Pixels,
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
    /// Input 公共 token。
    pub input: InputTokens,
    /// Checkbox 公共 token。
    pub checkbox: CheckboxTokens,
    /// Radio 公共 token。
    pub radio: RadioTokens,
    /// Select 公共 token。
    pub select: SelectTokens,
    /// Switch 公共 token。
    pub switch: SwitchTokens,
    /// Tooltip 公共 token。
    pub tooltip: TooltipTokens,
    /// Scrollbar 公共 token。
    pub scrollbar: ScrollbarTokens,
    input_states: [[InputStateTokens; 7]; 4],
    input_sizes: [InputSizeTokens; 4],
    select_trigger_states: [SelectTriggerStateTokens; 6],
    select_option_states: [SelectOptionStateTokens; 5],
    select_sizes: [SelectSizeTokens; 4],
    tokens: ResolvedTokens,
}

impl ResolvedTheme {
    /// 从已解析 DTCG token 构造 GPUI adapter 主题。
    pub fn from_tokens(
        mode: ResolvedThemeMode,
        tokens: ResolvedTokens,
    ) -> Result<Self, ThemeError> {
        let input_states = resolve_input_states(&tokens)?;
        let input_sizes = resolve_input_sizes(&tokens)?;
        let select_trigger_states = resolve_select_trigger_states(&tokens)?;
        let select_option_states = resolve_select_option_states(&tokens)?;
        let select_sizes = resolve_select_sizes(&tokens)?;
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
            input: InputTokens {
                border_width: dimension(&tokens, "input.border-width")?,
                focus_width: dimension(&tokens, "input.focus-width")?,
                caret_width: dimension(&tokens, "input.caret-width")?,
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
            radio: RadioTokens {
                border_width: optional_dimension(
                    &tokens,
                    "radio.border-width",
                    "foundation.border.width",
                )?,
                focus_width: optional_dimension(
                    &tokens,
                    "radio.focus-width",
                    "foundation.border.focus",
                )?,
            },
            select: SelectTokens {
                border_width: dimension(&tokens, "select.border-width")?,
                focus_width: dimension(&tokens, "select.focus-width")?,
                popup_background: color(&tokens, "select.popup.background")?,
                popup_border: color(&tokens, "select.popup.border")?,
                popup_border_width: dimension(&tokens, "select.popup.border-width")?,
                popup_radius: dimension(&tokens, "select.popup.radius")?,
                popup_padding: dimension(&tokens, "select.popup.padding")?,
                popup_shadow_color: color(&tokens, "select.popup.shadow-color")?,
                popup_shadow_offset_y: dimension(&tokens, "select.popup.shadow-offset-y")?,
                popup_shadow_blur: dimension(&tokens, "select.popup.shadow-blur")?,
                popup_shadow_spread: dimension(&tokens, "select.popup.shadow-spread")?,
                popup_anchor_gap: dimension(&tokens, "select.popup.anchor-gap")?,
                popup_viewport_padding: dimension(&tokens, "select.popup.viewport-padding")?,
                popup_max_height: dimension(&tokens, "select.popup.max-height")?,
                group_label: color(&tokens, "select.group-label")?,
                status_loading: color(&tokens, "select.status.loading")?,
                status_empty: color(&tokens, "select.status.empty")?,
                status_error: color(&tokens, "select.status.error")?,
            },
            switch: SwitchTokens {
                border_width: optional_dimension(
                    &tokens,
                    "switch.border-width",
                    "foundation.border.width",
                )?,
                focus_width: optional_dimension(
                    &tokens,
                    "switch.focus-width",
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
            scrollbar: ScrollbarTokens {
                track: optional_color(&tokens, "scrollbar.track", "semantic.secondary")?,
                thumb: optional_color(&tokens, "scrollbar.thumb", "semantic.on-muted")?,
                thumb_hover: optional_color(
                    &tokens,
                    "scrollbar.thumb-hover",
                    "semantic.foreground",
                )?,
                thumb_pressed: optional_color(
                    &tokens,
                    "scrollbar.thumb-pressed",
                    "semantic.foreground",
                )?,
                focus_ring: optional_color(&tokens, "scrollbar.focus-ring", "semantic.ring")?,
                thickness: optional_dimension_value(&tokens, "scrollbar.thickness", gpui::px(8.))?,
                thumb_hover_thickness: optional_dimension_value(
                    &tokens,
                    "scrollbar.thumb-hover-thickness",
                    gpui::px(10.),
                )?,
                hit_thickness: optional_dimension_value(
                    &tokens,
                    "scrollbar.hit-thickness",
                    gpui::px(14.),
                )?,
                min_thumb_length: optional_dimension_value(
                    &tokens,
                    "scrollbar.min-thumb-length",
                    gpui::px(24.),
                )?,
                radius: optional_dimension(&tokens, "scrollbar.radius", "foundation.radius.md")?,
                focus_width: optional_dimension(
                    &tokens,
                    "scrollbar.focus-width",
                    "foundation.border.focus",
                )?,
            },
            input_states,
            input_sizes,
            select_trigger_states,
            select_option_states,
            select_sizes,
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

    /// 读取已在构造阶段完整解析的 Input variant/state 样式 token。
    pub fn input_state(
        &self,
        variant: InputVariantKind,
        state: InputVisualState,
    ) -> InputStateTokens {
        self.input_states[variant as usize][state as usize]
    }

    /// 读取已在构造阶段完整解析的 Input size token。
    pub fn input_size(&self, size: ThemeSize) -> InputSizeTokens {
        self.input_sizes[size as usize]
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

    /// 读取 Radio 的选中/交互状态 token。
    pub fn radio_state(&self, selected: bool, state: &str) -> Result<RadioStateTokens, ThemeError> {
        let selection = if selected { "selected" } else { "unselected" };
        let prefix = format!("radio.state.{selection}.{state}");
        Ok(RadioStateTokens {
            background: optional_color(
                &self.tokens,
                &format!("{prefix}.background"),
                radio_background_fallback(state),
            )?,
            indicator_background: optional_color(
                &self.tokens,
                &format!("{prefix}.indicator-background"),
                radio_indicator_background_fallback(state),
            )?,
            border: optional_color(
                &self.tokens,
                &format!("{prefix}.border"),
                radio_border_fallback(selected, state),
            )?,
            dot: optional_color(
                &self.tokens,
                &format!("{prefix}.dot"),
                radio_dot_fallback(selected, state),
            )?,
            label: optional_color(
                &self.tokens,
                &format!("{prefix}.label"),
                checkbox_label_fallback(state),
            )?,
            description: optional_color(
                &self.tokens,
                &format!("{prefix}.description"),
                radio_description_fallback(state),
            )?,
        })
    }

    /// 读取 RadioGroup 的语义尺寸 token。
    pub fn radio_size(&self, size: &str) -> Result<RadioSizeTokens, ThemeError> {
        let prefix = format!("radio.size.{size}");
        Ok(RadioSizeTokens {
            indicator_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.indicator-size"),
                checkbox_box_size_fallback(size),
            )?,
            dot_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.dot-size"),
                radio_dot_size_fallback(size),
            )?,
            label_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.label-gap"),
                checkbox_label_gap_fallback(size),
            )?,
            description_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.description-gap"),
                "foundation.space.1",
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
            description_font_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.description-font-size"),
                "foundation.font.size.xs",
            )?,
            description_line_height: optional_dimension(
                &self.tokens,
                &format!("{prefix}.description-line-height"),
                "foundation.space.4",
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
            group_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.group-gap"),
                "foundation.space.1",
            )?,
        })
    }

    /// 读取已在构造阶段完整解析的 Select Trigger 状态 token。
    pub fn select_trigger_state(&self, state: SelectTriggerState) -> SelectTriggerStateTokens {
        self.select_trigger_states[state as usize]
    }

    /// 读取已在构造阶段完整解析的 Select Option 状态 token。
    pub fn select_option_state(&self, state: SelectOptionState) -> SelectOptionStateTokens {
        self.select_option_states[state as usize]
    }

    /// 读取已在构造阶段完整解析的 Select 尺寸 token。
    pub fn select_size(&self, size: ThemeSize) -> SelectSizeTokens {
        self.select_sizes[size as usize]
    }

    /// 读取 Switch state token。
    pub fn switch_state(
        &self,
        visual_state: &str,
        state: &str,
    ) -> Result<SwitchStateTokens, ThemeError> {
        let prefix = format!("switch.state.{visual_state}.{state}");
        Ok(SwitchStateTokens {
            track_background: optional_color(
                &self.tokens,
                &format!("{prefix}.track-background"),
                switch_track_background_fallback(visual_state, state),
            )?,
            track_border: optional_color(
                &self.tokens,
                &format!("{prefix}.track-border"),
                switch_track_border_fallback(visual_state, state),
            )?,
            thumb: optional_color(
                &self.tokens,
                &format!("{prefix}.thumb"),
                switch_thumb_fallback(visual_state, state),
            )?,
            label: optional_color(
                &self.tokens,
                &format!("{prefix}.label"),
                checkbox_label_fallback(state),
            )?,
            content: optional_color(
                &self.tokens,
                &format!("{prefix}.content"),
                switch_content_fallback(visual_state, state),
            )?,
            spinner: optional_color(
                &self.tokens,
                &format!("{prefix}.spinner"),
                switch_spinner_fallback(visual_state, state),
            )?,
        })
    }

    /// 读取 Switch size token。
    pub fn switch_size(&self, size: &str) -> Result<SwitchSizeTokens, ThemeError> {
        let prefix = format!("switch.size.{size}");
        let track_width = optional_dimension(
            &self.tokens,
            &format!("{prefix}.track-width"),
            switch_track_width_fallback(size),
        )?;
        let thumb_size = optional_dimension(
            &self.tokens,
            &format!("{prefix}.thumb-size"),
            switch_track_height_fallback(size),
        )?;
        let track_height = optional_dimension(
            &self.tokens,
            &format!("{prefix}.track-height"),
            switch_track_height_fallback(size),
        )?;
        let track_padding = optional_dimension(
            &self.tokens,
            &format!("{prefix}.track-padding"),
            "foundation.space.0",
        )?;
        let hit_size = optional_dimension(
            &self.tokens,
            &format!("{prefix}.hit-size"),
            switch_track_height_fallback(size),
        )?;
        Ok(SwitchSizeTokens {
            track_width,
            track_height,
            track_padding,
            thumb_size,
            track_radius: optional_dimension(
                &self.tokens,
                &format!("{prefix}.track-radius"),
                "foundation.radius.md",
            )?,
            thumb_radius: optional_dimension(
                &self.tokens,
                &format!("{prefix}.thumb-radius"),
                "foundation.radius.md",
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
            hit_size,
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
            content_track_height: if self
                .tokens
                .get(&format!("{prefix}.content-track-height"))
                .is_some()
            {
                dimension(&self.tokens, &format!("{prefix}.content-track-height"))?
            } else {
                hit_size
            },
            content_track_padding: if self
                .tokens
                .get(&format!("{prefix}.content-track-padding"))
                .is_some()
            {
                dimension(&self.tokens, &format!("{prefix}.content-track-padding"))?
            } else {
                track_padding
            },
            content_thumb_size: if self
                .tokens
                .get(&format!("{prefix}.content-thumb-size"))
                .is_some()
            {
                dimension(&self.tokens, &format!("{prefix}.content-thumb-size"))?
            } else {
                thumb_size
            },
            content_slot_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.content-slot-gap"),
                "foundation.space.1",
            )?,
            content_edge_padding: optional_dimension(
                &self.tokens,
                &format!("{prefix}.content-edge-padding"),
                "foundation.space.1",
            )?,
            content_icon_size: if self
                .tokens
                .get(&format!("{prefix}.content-icon-size"))
                .is_some()
            {
                dimension(&self.tokens, &format!("{prefix}.content-icon-size"))?
            } else {
                thumb_size
            },
            content_gap: optional_dimension(
                &self.tokens,
                &format!("{prefix}.content-gap"),
                "foundation.space.1",
            )?,
            content_max_text_width: if self
                .tokens
                .get(&format!("{prefix}.content-max-text-width"))
                .is_some()
            {
                dimension(&self.tokens, &format!("{prefix}.content-max-text-width"))?
            } else {
                track_width
            },
            spinner_size: optional_dimension(
                &self.tokens,
                &format!("{prefix}.spinner-size"),
                "foundation.space.2",
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

impl InputVariantKind {
    fn token(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Filled => "filled",
            Self::Borderless => "borderless",
            Self::Underline => "underline",
        }
    }
}

impl InputVisualState {
    fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::FocusVisible => "focus-visible",
            Self::Invalid => "invalid",
            Self::InvalidFocusVisible => "invalid-focus-visible",
            Self::ReadOnly => "read-only",
            Self::Disabled => "disabled",
        }
    }
}

impl ThemeSize {
    fn token(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

impl SelectTriggerState {
    fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::FocusVisible => "focus-visible",
            Self::Open => "open",
            Self::Disabled => "disabled",
        }
    }
}

impl SelectOptionState {
    fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::Active => "active",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
        }
    }
}

fn resolve_input_states(tokens: &ResolvedTokens) -> Result<[[InputStateTokens; 7]; 4], ThemeError> {
    Ok([
        resolve_input_variant(tokens, InputVariantKind::Outline)?,
        resolve_input_variant(tokens, InputVariantKind::Filled)?,
        resolve_input_variant(tokens, InputVariantKind::Borderless)?,
        resolve_input_variant(tokens, InputVariantKind::Underline)?,
    ])
}

fn resolve_input_variant(
    tokens: &ResolvedTokens,
    variant: InputVariantKind,
) -> Result<[InputStateTokens; 7], ThemeError> {
    Ok([
        resolve_input_state(tokens, variant, InputVisualState::Normal)?,
        resolve_input_state(tokens, variant, InputVisualState::Hover)?,
        resolve_input_state(tokens, variant, InputVisualState::FocusVisible)?,
        resolve_input_state(tokens, variant, InputVisualState::Invalid)?,
        resolve_input_state(tokens, variant, InputVisualState::InvalidFocusVisible)?,
        resolve_input_state(tokens, variant, InputVisualState::ReadOnly)?,
        resolve_input_state(tokens, variant, InputVisualState::Disabled)?,
    ])
}

fn resolve_input_state(
    tokens: &ResolvedTokens,
    variant: InputVariantKind,
    state: InputVisualState,
) -> Result<InputStateTokens, ThemeError> {
    let prefix = format!("input.variant.{}.{}", variant.token(), state.token());
    Ok(InputStateTokens {
        background: color(tokens, &format!("{prefix}.background"))?,
        foreground: color(tokens, &format!("{prefix}.foreground"))?,
        placeholder: color(tokens, &format!("{prefix}.placeholder"))?,
        border: color(tokens, &format!("{prefix}.border"))?,
        caret: color(tokens, &format!("{prefix}.caret"))?,
        selection: color(tokens, &format!("{prefix}.selection"))?,
        affix: color(tokens, &format!("{prefix}.affix"))?,
        status: color(tokens, &format!("{prefix}.status"))?,
    })
}

fn resolve_input_sizes(tokens: &ResolvedTokens) -> Result<[InputSizeTokens; 4], ThemeError> {
    Ok([
        resolve_input_size(tokens, ThemeSize::Xs)?,
        resolve_input_size(tokens, ThemeSize::Sm)?,
        resolve_input_size(tokens, ThemeSize::Md)?,
        resolve_input_size(tokens, ThemeSize::Lg)?,
    ])
}

fn resolve_input_size(
    tokens: &ResolvedTokens,
    size: ThemeSize,
) -> Result<InputSizeTokens, ThemeError> {
    let prefix = format!("input.size.{}", size.token());
    Ok(InputSizeTokens {
        height: dimension(tokens, &format!("{prefix}.height"))?,
        padding_x: dimension(tokens, &format!("{prefix}.padding-x"))?,
        font_size: dimension(tokens, &format!("{prefix}.font-size"))?,
        line_height: dimension(tokens, &format!("{prefix}.line-height"))?,
        radius: dimension(tokens, &format!("{prefix}.radius"))?,
        slot_size: dimension(tokens, &format!("{prefix}.slot-size"))?,
        icon_size: dimension(tokens, &format!("{prefix}.icon-size"))?,
        status_size: dimension(tokens, &format!("{prefix}.status-size"))?,
        gap: dimension(tokens, &format!("{prefix}.gap"))?,
    })
}

fn resolve_select_trigger_states(
    tokens: &ResolvedTokens,
) -> Result<[SelectTriggerStateTokens; 6], ThemeError> {
    Ok([
        resolve_select_trigger_state(tokens, SelectTriggerState::Normal)?,
        resolve_select_trigger_state(tokens, SelectTriggerState::Hover)?,
        resolve_select_trigger_state(tokens, SelectTriggerState::Pressed)?,
        resolve_select_trigger_state(tokens, SelectTriggerState::FocusVisible)?,
        resolve_select_trigger_state(tokens, SelectTriggerState::Open)?,
        resolve_select_trigger_state(tokens, SelectTriggerState::Disabled)?,
    ])
}

fn resolve_select_trigger_state(
    tokens: &ResolvedTokens,
    state: SelectTriggerState,
) -> Result<SelectTriggerStateTokens, ThemeError> {
    let prefix = format!("select.trigger.{}", state.token());
    Ok(SelectTriggerStateTokens {
        background: color(tokens, &format!("{prefix}.background"))?,
        foreground: color(tokens, &format!("{prefix}.foreground"))?,
        placeholder: color(tokens, &format!("{prefix}.placeholder"))?,
        border: color(tokens, &format!("{prefix}.border"))?,
        indicator: color(tokens, &format!("{prefix}.indicator"))?,
    })
}

fn resolve_select_option_states(
    tokens: &ResolvedTokens,
) -> Result<[SelectOptionStateTokens; 5], ThemeError> {
    Ok([
        resolve_select_option_state(tokens, SelectOptionState::Normal)?,
        resolve_select_option_state(tokens, SelectOptionState::Hover)?,
        resolve_select_option_state(tokens, SelectOptionState::Active)?,
        resolve_select_option_state(tokens, SelectOptionState::Selected)?,
        resolve_select_option_state(tokens, SelectOptionState::Disabled)?,
    ])
}

fn resolve_select_option_state(
    tokens: &ResolvedTokens,
    state: SelectOptionState,
) -> Result<SelectOptionStateTokens, ThemeError> {
    let prefix = format!("select.option.{}", state.token());
    Ok(SelectOptionStateTokens {
        background: color(tokens, &format!("{prefix}.background"))?,
        foreground: color(tokens, &format!("{prefix}.foreground"))?,
        description: color(tokens, &format!("{prefix}.description"))?,
        indicator: color(tokens, &format!("{prefix}.indicator"))?,
    })
}

fn resolve_select_sizes(tokens: &ResolvedTokens) -> Result<[SelectSizeTokens; 4], ThemeError> {
    Ok([
        resolve_select_size(tokens, ThemeSize::Xs)?,
        resolve_select_size(tokens, ThemeSize::Sm)?,
        resolve_select_size(tokens, ThemeSize::Md)?,
        resolve_select_size(tokens, ThemeSize::Lg)?,
    ])
}

fn resolve_select_size(
    tokens: &ResolvedTokens,
    size: ThemeSize,
) -> Result<SelectSizeTokens, ThemeError> {
    let prefix = format!("select.size.{}", size.token());
    Ok(SelectSizeTokens {
        height: dimension(tokens, &format!("{prefix}.height"))?,
        padding_x: dimension(tokens, &format!("{prefix}.padding-x"))?,
        font_size: dimension(tokens, &format!("{prefix}.font-size"))?,
        line_height: dimension(tokens, &format!("{prefix}.line-height"))?,
        radius: dimension(tokens, &format!("{prefix}.radius"))?,
        icon_size: dimension(tokens, &format!("{prefix}.icon-size"))?,
        indicator_size: dimension(tokens, &format!("{prefix}.indicator-size"))?,
        content_gap: dimension(tokens, &format!("{prefix}.content-gap"))?,
        option_padding_x: dimension(tokens, &format!("{prefix}.option-padding-x"))?,
        option_padding_y: dimension(tokens, &format!("{prefix}.option-padding-y"))?,
        description_font_size: dimension(tokens, &format!("{prefix}.description-font-size"))?,
        description_line_height: dimension(tokens, &format!("{prefix}.description-line-height"))?,
        group_padding_y: dimension(tokens, &format!("{prefix}.group-padding-y"))?,
    })
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

fn optional_dimension_value(
    tokens: &ResolvedTokens,
    path: &str,
    fallback: Pixels,
) -> Result<Pixels, ThemeError> {
    if tokens.get(path).is_some() {
        dimension(tokens, path)
    } else {
        Ok(fallback)
    }
}

fn checkbox_box_background_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-background"
    } else if visual_state == "unchecked" {
        "semantic.background"
    } else {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    }
}

fn checkbox_border_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "focus-visible" {
        "semantic.ring"
    } else if state == "disabled" {
        "semantic.disabled-border"
    } else if visual_state == "unchecked" {
        if matches!(state, "hover" | "pressed") {
            "semantic.border"
        } else {
            "semantic.input-border"
        }
    } else {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
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

fn radio_background_fallback(state: &str) -> &'static str {
    match state {
        "hover" => "semantic.accent",
        "pressed" => "semantic.accent-pressed",
        _ => "semantic.background",
    }
}

fn radio_indicator_background_fallback(state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-background"
    } else {
        "semantic.background"
    }
}

fn radio_border_fallback(selected: bool, state: &str) -> &'static str {
    if state == "focus-visible" {
        "semantic.ring"
    } else if state == "disabled" {
        "semantic.disabled-border"
    } else if selected {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    } else if matches!(state, "hover" | "pressed") {
        "semantic.border"
    } else {
        "semantic.input-border"
    }
}

fn radio_dot_fallback(selected: bool, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else if selected {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    } else {
        "semantic.background"
    }
}

fn radio_description_fallback(state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else if state == "pressed" {
        "semantic.foreground"
    } else {
        "semantic.on-muted"
    }
}

fn radio_dot_size_fallback(size: &str) -> &'static str {
    match size {
        "xs" => "foundation.space.1",
        "sm" => "foundation.space.1_5",
        "md" => "foundation.space.2",
        "lg" => "foundation.space.2_5",
        _ => "foundation.space.2",
    }
}

fn switch_track_background_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-background"
    } else if visual_state == "checked" {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    } else {
        match state {
            "hover" => "semantic.border",
            "pressed" => "semantic.accent-pressed",
            _ => "semantic.input-border",
        }
    }
}

fn switch_track_border_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "focus-visible" {
        "semantic.ring"
    } else if state == "disabled" {
        "semantic.disabled-border"
    } else if visual_state == "checked" {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    } else if matches!(state, "hover" | "pressed") {
        "semantic.border"
    } else {
        "semantic.input-border"
    }
}

fn switch_thumb_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else if visual_state == "checked" {
        "semantic.on-primary"
    } else {
        "semantic.background"
    }
}

fn switch_content_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.disabled-foreground"
    } else if visual_state == "checked" {
        "semantic.on-primary"
    } else {
        "semantic.foreground"
    }
}

fn switch_spinner_fallback(visual_state: &str, state: &str) -> &'static str {
    if state == "disabled" {
        "semantic.background"
    } else if visual_state == "checked" {
        match state {
            "hover" => "semantic.primary-hover",
            "pressed" => "semantic.primary-pressed",
            _ => "semantic.primary",
        }
    } else {
        "semantic.foreground"
    }
}

fn switch_track_width_fallback(size: &str) -> &'static str {
    let _ = size;
    "foundation.space.4"
}

fn switch_track_height_fallback(size: &str) -> &'static str {
    let _ = size;
    "foundation.space.3"
}
