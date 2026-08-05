//! 支持 IME 的单行文本 Input 组件。

use crate::{
    ComponentSize, Icon, IconButton, IconButtonVariant, IconSource, Tooltip, TooltipPlacement,
    component_size, theme,
    traits::{Changeable, Disableable, Focusable, Sizable},
};
use gpui::{
    A11ySubtreeBuilder, AccessibleAction, AnyElement, App, Bounds, ClipboardItem, ContentMask,
    Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    EventEmitter, FocusHandle, GlobalElementId, Hsla, InspectorElementId, InteractiveElement,
    IntoElement, KeyDownEvent, LayoutId, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, ParentElement, Pixels, Point, Render, RenderOnce, Role, ShapedLine,
    SharedString, StatefulInteractiveElement, Style, Styled, Subscription, Task, TextAlign,
    TextRun, UTF16Selection, UnderlineStyle, Window,
    accesskit::{self, ActionData},
    div, fill, point,
    prelude::FluentBuilder,
    px, relative, size,
};
use std::{ops::Range, rc::Rc, time::Duration};
use unicode_segmentation::UnicodeSegmentation as _;
use vektra_theme::{InputSizeTokens, InputStateTokens, ResolvedTheme};

const MAX_HISTORY_ENTRIES: usize = 100;
const MAX_CHARS_PER_TEXT_RUN: usize = 255;
const CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);
const PASSWORD_MASK: char = '•';

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type SubmitHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Input 的视觉语义变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputVariant {
    /// 完整边框，适合普通表单输入。
    #[default]
    Outline,
    /// 填充背景并保留稳定边框几何。
    Filled,
    /// 静止时无明显背景和边框，仍保留焦点与错误反馈。
    Borderless,
    /// 仅使用底线表达边界和交互状态。
    Underline,
}

impl InputVariant {
    const fn token_key(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Filled => "filled",
            Self::Borderless => "borderless",
            Self::Underline => "underline",
        }
    }
}

/// Input 的单行文本输入语义。
///
/// 该类型只决定无障碍角色和 Password 的安全显示行为，不会自动添加图标、清除操作、
/// 格式化、字符过滤或业务校验。Email、Phone 与 Url 的合法性仍由宿主应用负责。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputType {
    /// 普通单行文本输入。
    #[default]
    Text,
    /// 搜索条件输入。
    Search,
    /// 密码输入；默认按 grapheme 使用固定字符掩码显示。
    Password,
    /// 电子邮箱输入，仅提供语义角色，不执行自动校验。
    Email,
    /// 电话号码输入，仅提供语义角色，不执行格式化或字符过滤。
    Phone,
    /// URL 输入，仅提供语义角色，不执行自动校验。
    Url,
}

impl InputType {
    const fn role(self) -> Role {
        match self {
            Self::Text => Role::TextInput,
            Self::Search => Role::SearchInput,
            Self::Password => Role::PasswordInput,
            Self::Email => Role::EmailInput,
            Self::Phone => Role::PhoneNumberInput,
            Self::Url => Role::UrlInput,
        }
    }
}

/// InputState 对外发布的语义事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// 用户编辑使值实际发生变化。
    Changed(SharedString),
    /// 文本编辑器实际获得焦点。
    Focused,
    /// 文本编辑器实际失去焦点。
    Blurred,
    /// 非 IME 组合状态下按下 Enter。
    Submitted(SharedString),
}

/// Input 内置清除操作的语义配置。
///
/// `aria_label` 是纯图标按钮的必需可访问名称；Tooltip 是独立的视觉帮助，Vektra
/// 不会在两者之间自动复制文本。
#[derive(Clone)]
pub struct InputClear {
    aria_label: SharedString,
    tooltip: Option<Tooltip>,
    tooltip_placement: TooltipPlacement,
}

impl InputClear {
    /// 创建带必需可访问名称的清除操作。
    pub fn new(aria_label: impl Into<SharedString>) -> Self {
        Self {
            aria_label: aria_label.into(),
            tooltip: None,
            tooltip_placement: TooltipPlacement::default(),
        }
    }

    /// 设置清除按钮的 Tooltip。
    pub fn tooltip(mut self, tooltip: impl Into<Tooltip>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// 设置 Tooltip 相对清除按钮的优先位置。
    pub fn tooltip_placement(mut self, placement: TooltipPlacement) -> Self {
        self.tooltip_placement = placement;
        self
    }
}

#[derive(Clone)]
struct EditSnapshot {
    value: String,
    selection: Range<usize>,
    selection_reversed: bool,
}

#[derive(Clone)]
struct InputRuntime {
    id: ElementId,
    placeholder: SharedString,
    variant: InputVariant,
    input_type: InputType,
    password_revealed: bool,
    size: ComponentSize,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    caret_color: Option<Hsla>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl Default for InputRuntime {
    fn default() -> Self {
        Self {
            id: ElementId::from("input"),
            placeholder: SharedString::default(),
            variant: InputVariant::default(),
            input_type: InputType::default(),
            password_revealed: false,
            size: ComponentSize::default(),
            disabled: false,
            read_only: false,
            invalid: false,
            caret_color: None,
            aria_label: None,
            aria_description: None,
            on_change: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }
    }
}

impl InputRuntime {
    fn password_hidden(&self) -> bool {
        self.input_type == InputType::Password && !self.password_revealed
    }
}

#[derive(Clone)]
struct DisplayText {
    text: String,
    masked: bool,
    real_boundaries: Vec<usize>,
    display_boundaries: Vec<usize>,
}

impl DisplayText {
    fn new(value: &str, password_hidden: bool) -> Self {
        let real_boundaries = value
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .chain(std::iter::once(value.len()))
            .collect::<Vec<_>>();
        if !password_hidden {
            return Self {
                text: value.to_owned(),
                masked: false,
                display_boundaries: real_boundaries.clone(),
                real_boundaries,
            };
        }

        let grapheme_count = real_boundaries.len().saturating_sub(1);
        let text = std::iter::repeat_n(PASSWORD_MASK, grapheme_count).collect::<String>();
        let mask_len = PASSWORD_MASK.len_utf8();
        let display_boundaries = (0..=grapheme_count).map(|index| index * mask_len).collect();
        Self {
            text,
            masked: true,
            real_boundaries,
            display_boundaries,
        }
    }

    fn display_offset(&self, real_offset: usize) -> usize {
        if !self.masked {
            return real_offset.min(self.text.len());
        }
        nearest_paired_boundary(real_offset, &self.real_boundaries, &self.display_boundaries)
    }

    fn real_offset(&self, display_offset: usize) -> usize {
        if !self.masked {
            return display_offset.min(self.text.len());
        }
        nearest_paired_boundary(
            display_offset,
            &self.display_boundaries,
            &self.real_boundaries,
        )
    }

    fn display_range(&self, range: Range<usize>) -> Range<usize> {
        self.display_offset(range.start)..self.display_offset(range.end)
    }
}

/// 调用方持有的单行文本编辑状态。
///
/// `InputState` 管理文本、UTF-8/UTF-16 索引、选区、IME marked range、撤销历史、
/// 焦点、布局与水平滚动。required、错误消息、dirty、touched 等表单元数据应由未来的
/// 外围表单容器持有，不属于本类型。
pub struct InputState {
    value: String,
    selection: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    composition_snapshot: Option<EditSnapshot>,
    undo_stack: Vec<EditSnapshot>,
    redo_stack: Vec<EditSnapshot>,
    focus_handle: FocusHandle,
    focus_subscription: Option<Subscription>,
    blur_subscription: Option<Subscription>,
    last_layout: Option<ShapedLine>,
    last_display: Option<DisplayText>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll_x: Pixels,
    is_selecting: bool,
    editor_focused: bool,
    caret_phase_visible: bool,
    caret_blink_generation: u64,
    caret_blink_task: Option<Task<()>>,
    #[cfg(test)]
    last_caret: Option<(Bounds<Pixels>, f32)>,
    #[cfg(test)]
    last_caret_blinking: bool,
    runtime: InputRuntime,
}

impl InputState {
    /// 创建由调用方持有的 Input 编辑状态。
    pub fn new(initial_value: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let value = normalize_single_line(initial_value.into().as_ref());
        let caret = value.len();
        Self {
            value,
            selection: caret..caret,
            selection_reversed: false,
            marked_range: None,
            composition_snapshot: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            focus_handle: cx.focus_handle(),
            focus_subscription: None,
            blur_subscription: None,
            last_layout: None,
            last_display: None,
            last_bounds: None,
            scroll_x: Pixels::ZERO,
            is_selecting: false,
            editor_focused: false,
            caret_phase_visible: true,
            caret_blink_generation: 0,
            caret_blink_task: None,
            #[cfg(test)]
            last_caret: None,
            #[cfg(test)]
            last_caret_blinking: false,
            runtime: InputRuntime::default(),
        }
    }

    /// 返回当前文本值。
    ///
    /// IME preedit 期间返回包含当前组合文本的即时值；是否已成为用户语义变更应以
    /// [`InputEvent::Changed`] 为准。
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 程序化替换文本，不发送 [`InputEvent::Changed`]。
    ///
    /// 该操作会安全结束 IME 组合，把光标放在新值末尾，并保留可供后续 undo 的既有
    /// 编辑历史。换行会转换为空格，文本不会被 trim。
    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        let value = normalize_single_line(value.into().as_ref());
        if self.value == value && self.marked_range.is_none() {
            return;
        }
        if self.value != value {
            let before = self
                .composition_snapshot
                .clone()
                .unwrap_or_else(|| self.snapshot());
            self.push_undo(before);
            self.redo_stack.clear();
        }
        self.value = value;
        let caret = self.value.len();
        self.selection = caret..caret;
        self.selection_reversed = false;
        self.end_composition();
        self.restart_caret_blink(cx);
        cx.notify();
    }

    /// 程序化清空文本，不发送 [`InputEvent::Changed`]。
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_value(SharedString::default(), cx);
    }

    /// 程序化重置文本、选区、IME、撤销/重做历史和水平滚动。
    ///
    /// 该操作不发送 [`InputEvent::Changed`]。
    pub fn reset(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.value = normalize_single_line(value.into().as_ref());
        let caret = self.value.len();
        self.selection = caret..caret;
        self.selection_reversed = false;
        self.end_composition();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.scroll_x = Pixels::ZERO;
        self.last_layout = None;
        self.last_display = None;
        self.last_bounds = None;
        self.is_selecting = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    /// 选择全部当前文本，不改变值且不发送 [`InputEvent::Changed`]。
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        if self.runtime.disabled {
            return;
        }
        self.selection = 0..self.value.len();
        self.selection_reversed = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    /// 返回文本编辑器使用的 GPUI 焦点句柄。
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn configure(&mut self, runtime: InputRuntime, window: &mut Window, cx: &mut Context<Self>) {
        let became_disabled = !self.runtime.disabled && runtime.disabled;
        let caret_mode_changed = self.runtime.disabled != runtime.disabled
            || self.runtime.read_only != runtime.read_only;
        let input_type_changed = self.runtime.input_type != runtime.input_type;
        let password_display_changed = self.runtime.password_hidden() != runtime.password_hidden();
        self.runtime = runtime;
        if input_type_changed || password_display_changed {
            self.last_layout = None;
            self.last_display = None;
            cx.notify();
        }
        if caret_mode_changed {
            self.restart_caret_blink(cx);
        }
        self.ensure_focus_subscriptions(window, cx);
        if became_disabled && self.focus_handle.is_focused(window) {
            window.blur();
        }
    }

    fn ensure_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_subscription.is_none() {
            let handle = self.focus_handle.clone();
            self.focus_subscription = Some(cx.on_focus(&handle, window, |this, window, cx| {
                this.editor_focused = true;
                this.restart_caret_blink(cx);
                this.emit_event(InputEvent::Focused, window, cx);
            }));
        }
        if self.blur_subscription.is_none() {
            let handle = self.focus_handle.clone();
            self.blur_subscription = Some(cx.on_blur(&handle, window, |this, window, cx| {
                this.editor_focused = false;
                this.restart_caret_blink(cx);
                this.emit_event(InputEvent::Blurred, window, cx);
            }));
        }
    }

    fn snapshot(&self) -> EditSnapshot {
        EditSnapshot {
            value: self.value.clone(),
            selection: self.selection.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    fn push_undo(&mut self, snapshot: EditSnapshot) {
        if self.undo_stack.len() == MAX_HISTORY_ENTRIES {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snapshot);
    }

    fn push_redo(&mut self, snapshot: EditSnapshot) {
        if self.redo_stack.len() == MAX_HISTORY_ENTRIES {
            self.redo_stack.remove(0);
        }
        self.redo_stack.push(snapshot);
    }

    fn apply_snapshot(
        &mut self,
        snapshot: EditSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let changed = self.value != snapshot.value;
        self.value = snapshot.value;
        self.selection = normalize_selection(&self.value, snapshot.selection);
        self.selection_reversed = snapshot.selection_reversed && !self.selection.is_empty();
        self.end_composition();
        self.restart_caret_blink(cx);
        if changed {
            self.emit_changed(window, cx);
        } else {
            cx.notify();
        }
    }

    fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        let Some(snapshot) = self.undo_stack.pop() else {
            return;
        };
        self.push_redo(self.snapshot());
        self.apply_snapshot(snapshot, window, cx);
    }

    fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        let Some(snapshot) = self.redo_stack.pop() else {
            return;
        };
        self.push_undo(self.snapshot());
        self.apply_snapshot(snapshot, window, cx);
    }

    fn can_edit(&self) -> bool {
        !self.runtime.disabled && !self.runtime.read_only
    }

    fn display_text(&self) -> DisplayText {
        DisplayText::new(&self.value, self.runtime.password_hidden())
    }

    fn end_composition(&mut self) {
        self.marked_range = None;
        self.composition_snapshot = None;
    }

    fn restart_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_blink_generation = self.caret_blink_generation.wrapping_add(1);
        self.caret_blink_task = None;
        self.caret_phase_visible = true;
        self.sync_caret_blink(cx);
    }

    fn sync_caret_blink(&mut self, cx: &mut Context<Self>) {
        let should_blink = caret_should_blink(
            caret_is_visible(
                self.editor_focused,
                self.selection.is_empty(),
                self.runtime.disabled,
                self.runtime.read_only,
            ),
            self.marked_range.is_some(),
            cx.reduce_motion(),
        );
        if !should_blink {
            if self.caret_blink_task.take().is_some() {
                self.caret_blink_generation = self.caret_blink_generation.wrapping_add(1);
            }
            self.caret_phase_visible = true;
            return;
        }
        if self.caret_blink_task.is_some() {
            return;
        }

        let generation = self.caret_blink_generation;
        self.caret_blink_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(CARET_BLINK_INTERVAL).await;
                let keep_blinking = this
                    .update(cx, |this, cx| {
                        if this.caret_blink_generation != generation {
                            return false;
                        }
                        this.caret_phase_visible = !this.caret_phase_visible;
                        cx.notify();
                        true
                    })
                    .unwrap_or(false);
                if !keep_blinking {
                    break;
                }
            }
        }));
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    fn anchor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selection.end
        } else {
            self.selection.start
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(&self.value, offset);
        self.selection = offset..offset;
        self.selection_reversed = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(&self.value, offset);
        let anchor = self.anchor_offset();
        if offset < anchor {
            self.selection = offset..anchor;
            self.selection_reversed = true;
        } else {
            self.selection = anchor..offset;
            self.selection_reversed = false;
        }
        self.restart_caret_blink(cx);
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        previous_grapheme_boundary(&self.value, offset)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        next_grapheme_boundary(&self.value, offset)
    }

    fn previous_word_boundary(&self, offset: usize) -> usize {
        previous_word_boundary(&self.value, offset)
    }

    fn next_word_boundary(&self, offset: usize) -> usize {
        next_word_boundary(&self.value, offset)
    }

    fn replace_user_range(
        &mut self,
        range: Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        let range = normalize_selection(&self.value, range);
        let text = normalize_single_line(text);
        let next = format!(
            "{}{}{}",
            &self.value[..range.start],
            text,
            &self.value[range.end..]
        );
        if next == self.value {
            self.selection = range.start + text.len()..range.start + text.len();
            self.selection_reversed = false;
            self.end_composition();
            self.restart_caret_blink(cx);
            cx.notify();
            return;
        }
        self.push_undo(self.snapshot());
        self.redo_stack.clear();
        self.value = next;
        let caret = range.start + text.len();
        self.selection = caret..caret;
        self.selection_reversed = false;
        self.end_composition();
        self.restart_caret_blink(cx);
        self.emit_changed(window, cx);
    }

    fn user_clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.value.is_empty() || !self.can_edit() {
            return;
        }
        let len = self.value.len();
        self.replace_user_range(0..len, "", window, cx);
    }

    fn emit_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value: SharedString = self.value.clone().into();
        self.emit_event(InputEvent::Changed(value), window, cx);
    }

    fn emit_event(&mut self, event: InputEvent, window: &mut Window, cx: &mut Context<Self>) {
        match &event {
            InputEvent::Changed(value) => {
                if let Some(handler) = self.runtime.on_change.clone() {
                    handler(value.clone(), window, cx);
                }
            }
            InputEvent::Submitted(value) => {
                if let Some(handler) = self.runtime.on_submit.clone() {
                    handler(value.clone(), window, cx);
                }
            }
            InputEvent::Focused => {
                if let Some(handler) = self.runtime.on_focus.clone() {
                    handler(window, cx);
                }
            }
            InputEvent::Blurred => {
                if let Some(handler) = self.runtime.on_blur.clone() {
                    handler(window, cx);
                }
            }
        }
        cx.emit(event);
        cx.notify();
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.value.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line), Some(display)) = (
            self.last_bounds,
            self.last_layout.as_ref(),
            self.last_display.as_ref(),
        ) else {
            return 0;
        };
        let display_index = if position.x <= bounds.left() {
            0
        } else if position.x >= bounds.right() {
            display.text.len()
        } else {
            line.closest_index_for_x(position.x - bounds.left() + self.scroll_x)
        };
        display.real_offset(display_index)
    }

    fn select_word_at(&mut self, offset: usize, cx: &mut Context<Self>) {
        let range = word_range_at(&self.value, offset);
        self.selection = range;
        self.selection_reversed = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.runtime.disabled {
            return;
        }
        window.focus(&self.focus_handle, cx);
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        match event.click_count {
            3.. => self.select_all(cx),
            2 => self.select_word_at(offset, cx),
            _ if event.modifiers.shift => self.select_to(offset, cx),
            _ => self.move_to(offset, cx),
        }
        window.prevent_default();
        cx.stop_propagation();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting && !self.runtime.disabled {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.runtime.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let secondary = secondary_shortcut_modifiers(modifiers, false);
        let secondary_shift = secondary_shortcut_modifiers(modifiers, true);

        if secondary && key == "a" {
            self.select_all(cx);
            stop_key_event(window, cx);
            return;
        }
        if secondary && key == "c" {
            self.copy_selection(cx);
            stop_key_event(window, cx);
            return;
        }
        if secondary && key == "x" {
            if self.runtime.password_hidden() {
                stop_key_event(window, cx);
            } else if self.can_edit() {
                self.cut_selection(window, cx);
                stop_key_event(window, cx);
            }
            return;
        }
        if secondary && key == "v" && self.can_edit() {
            #[cfg(not(target_family = "wasm"))]
            {
                self.paste(window, cx);
                stop_key_event(window, cx);
            }
            return;
        }
        if secondary && key == "z" && self.can_edit() {
            self.undo(window, cx);
            stop_key_event(window, cx);
            return;
        }
        if ((secondary_shift && key == "z") || (secondary && key == "y")) && self.can_edit() {
            self.redo(window, cx);
            stop_key_event(window, cx);
            return;
        }
        if key == "enter" && plain_or_shift_modifiers(modifiers) {
            if self.marked_range.is_none() {
                self.emit_event(InputEvent::Submitted(self.value.clone().into()), window, cx);
                stop_key_event(window, cx);
            }
            return;
        }
        if key == "left"
            && let Some(kind) = horizontal_movement_kind(modifiers)
        {
            self.move_horizontal(false, kind, modifiers.shift, cx);
            stop_key_event(window, cx);
            return;
        }
        if key == "right"
            && let Some(kind) = horizontal_movement_kind(modifiers)
        {
            self.move_horizontal(true, kind, modifiers.shift, cx);
            stop_key_event(window, cx);
            return;
        }
        if key == "home" && plain_or_shift_modifiers(modifiers) {
            if modifiers.shift {
                self.select_to(0, cx);
            } else {
                self.move_to(0, cx);
            }
            stop_key_event(window, cx);
            return;
        }
        if key == "end" && plain_or_shift_modifiers(modifiers) {
            let end = self.value.len();
            if modifiers.shift {
                self.select_to(end, cx);
            } else {
                self.move_to(end, cx);
            }
            stop_key_event(window, cx);
            return;
        }
        if key == "backspace"
            && self.can_edit()
            && let Some(kind) = deletion_kind(modifiers)
        {
            self.delete_backward(kind, window, cx);
            stop_key_event(window, cx);
            return;
        }
        if key == "delete"
            && self.can_edit()
            && let Some(kind) = deletion_kind(modifiers)
        {
            self.delete_forward(kind, window, cx);
            stop_key_event(window, cx);
        }
    }

    fn move_horizontal(
        &mut self,
        right: bool,
        kind: HorizontalMovement,
        extend_selection: bool,
        cx: &mut Context<Self>,
    ) {
        let cursor = self.cursor_offset();
        let target = horizontal_target(self, right, kind);
        if extend_selection {
            self.select_to(target, cx);
        } else if !self.selection.is_empty() && kind == HorizontalMovement::Grapheme {
            self.move_to(
                if right {
                    self.selection.end
                } else {
                    self.selection.start
                },
                cx,
            );
        } else if target != cursor || self.selection.is_empty() {
            self.move_to(target, cx);
        }
    }

    fn delete_backward(&mut self, kind: Deletion, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            self.replace_user_range(self.selection.clone(), "", window, cx);
            return;
        }
        let cursor = self.cursor_offset();
        let start = match kind {
            Deletion::Grapheme => self.previous_boundary(cursor),
            Deletion::Word => self.previous_word_boundary(cursor),
            #[cfg(target_os = "macos")]
            Deletion::Line => 0,
        };
        self.replace_user_range(start..cursor, "", window, cx);
    }

    fn delete_forward(&mut self, kind: Deletion, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            self.replace_user_range(self.selection.clone(), "", window, cx);
            return;
        }
        let cursor = self.cursor_offset();
        let end = match kind {
            Deletion::Grapheme => self.next_boundary(cursor),
            Deletion::Word => self.next_word_boundary(cursor),
            #[cfg(target_os = "macos")]
            Deletion::Line => self.value.len(),
        };
        self.replace_user_range(cursor..end, "", window, cx);
    }

    fn copy_selection(&self, cx: &mut Context<Self>) {
        if !self.runtime.password_hidden() && !self.selection.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.value[self.selection.clone()].to_owned(),
            ));
        }
    }

    fn cut_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.runtime.password_hidden() || self.selection.is_empty() || !self.can_edit() {
            return;
        }
        self.copy_selection(cx);
        self.replace_user_range(self.selection.clone(), "", window, cx);
    }

    #[cfg(not(target_family = "wasm"))]
    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_user_range(self.selection.clone(), &text, window, cx);
        }
    }

    fn a11y_state(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (String, impl FnOnce(&mut A11ySubtreeBuilder) + 'static) {
        let state = window.is_a11y_active().then(|| {
            let display = self.display_text();
            let selection = display.display_range(self.selection.clone());
            let text = display.text;
            let selection_reversed = self.selection_reversed;
            let focused = self.focus_handle.is_focused(window);
            let password_hidden = self.runtime.password_hidden();
            let value_state_key = if password_hidden {
                "input-a11y-password-value"
            } else {
                "input-a11y-value"
            };
            let value_state =
                window.use_keyed_state((self.runtime.id.clone(), value_state_key), cx, {
                    let text = text.clone();
                    move |_, _| text
                });
            if (password_hidden || !focused) && *value_state.read(cx) != text {
                *value_state.as_mut(cx) = text.clone();
            }
            let frozen_value = value_state.read(cx).clone();
            let (selection_tail, selection_head) = if selection_reversed {
                (selection.end, selection.start)
            } else {
                (selection.start, selection.end)
            };
            (
                frozen_value,
                text,
                selection_tail,
                selection_head,
                self.runtime.invalid,
                self.runtime.read_only,
                self.runtime.disabled,
            )
        });
        let (frozen_value, run_data) = match state {
            Some((
                frozen_value,
                text,
                selection_tail,
                selection_head,
                invalid,
                read_only,
                disabled,
            )) => (
                frozen_value,
                Some((
                    text,
                    selection_tail,
                    selection_head,
                    invalid,
                    read_only,
                    disabled,
                )),
            ),
            None => (String::new(), None),
        };

        let text_runs = move |builder: &mut A11ySubtreeBuilder| {
            if let Some((text, selection_tail, selection_head, invalid, read_only, disabled)) =
                run_data
            {
                push_a11y_text_runs(builder, &text, selection_tail, selection_head);
                let node = builder.parent_node();
                if invalid {
                    node.set_invalid(accesskit::Invalid::True);
                }
                if read_only {
                    node.set_read_only();
                }
                if disabled {
                    node.set_disabled();
                }
            }
        };
        (frozen_value, text_runs)
    }
}

impl EventEmitter<InputEvent> for InputState {}

impl EntityInputHandler for InputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = range_from_utf16(&self.value, range_utf16);
        adjusted_range.replace(range_to_utf16(&self.value, range.clone()));
        Some(self.value[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.runtime.disabled && !ignore_disabled_input {
            return None;
        }
        Some(UTF16Selection {
            range: range_to_utf16(&self.value, self.selection.clone()),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .clone()
            .map(|range| range_to_utf16(&self.value, range))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let changed = self
            .composition_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.value != self.value);
        if let Some(snapshot) = self.composition_snapshot.take()
            && changed
        {
            self.push_undo(snapshot);
            self.redo_stack.clear();
        }
        self.marked_range = None;
        self.restart_caret_blink(cx);
        if changed {
            self.emit_changed(window, cx);
        } else {
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        let range = range_utf16
            .map(|range| range_from_utf16(&self.value, range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selection.clone());
        let composition_snapshot = self.composition_snapshot.take();
        let before = composition_snapshot
            .clone()
            .unwrap_or_else(|| self.snapshot());
        let range = normalize_selection(&self.value, range);
        let text = normalize_single_line(text);
        let next = format!(
            "{}{}{}",
            &self.value[..range.start],
            text,
            &self.value[range.end..]
        );
        self.value = next;
        let caret = range.start + text.len();
        self.selection = caret..caret;
        self.selection_reversed = false;
        self.marked_range = None;
        self.restart_caret_blink(cx);
        if self.value != before.value {
            self.push_undo(before);
            self.redo_stack.clear();
            self.emit_changed(window, cx);
        } else {
            cx.notify();
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        if self.composition_snapshot.is_none() {
            self.composition_snapshot = Some(self.snapshot());
        }
        let range = range_utf16
            .map(|range| range_from_utf16(&self.value, range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selection.clone());
        let range = normalize_selection(&self.value, range);
        let new_text = normalize_single_line(new_text);
        self.value = format!(
            "{}{}{}",
            &self.value[..range.start],
            new_text,
            &self.value[range.end..]
        );
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        self.selection = new_selected_range_utf16
            .map(|selection| range_from_utf16(&new_text, selection))
            .map(|selection| range.start + selection.start..range.start + selection.end)
            .unwrap_or_else(|| {
                let caret = range.start + new_text.len();
                caret..caret
            });
        self.selection_reversed = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let display = self.last_display.as_ref()?;
        let range = display.display_range(range_from_utf16(&self.value, range_utf16));
        Some(Bounds::from_corners(
            point(
                bounds.left() + line.x_for_index(range.start) - self.scroll_x,
                bounds.top(),
            ),
            point(
                bounds.left() + line.x_for_index(range.end) - self.scroll_x,
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(utf8_to_utf16(
            &self.value,
            self.index_for_mouse_position(point),
        ))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.runtime.disabled {
            return;
        }
        self.selection = range_from_utf16(&self.value, range_utf16);
        self.selection_reversed = false;
        self.restart_caret_blink(cx);
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.value.encode_utf16().count())
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.can_edit()
    }
}

impl Render for InputState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_focus_subscriptions(window, cx);
        let theme = theme::current_theme(window, cx);
        let size = theme
            .input_size(self.runtime.size.token_key())
            .expect("Vektra 默认 Input size token 必须通过测试保持有效");
        let editor_focused = self.focus_handle.is_focused(window);
        if self.editor_focused != editor_focused {
            self.editor_focused = editor_focused;
            self.restart_caret_blink(cx);
        } else {
            self.sync_caret_blink(cx);
        }
        let focus_visible = editor_focused && window.last_input_was_keyboard();
        let state_key = if self.runtime.disabled {
            "disabled"
        } else if self.runtime.invalid && focus_visible {
            "invalid-focus-visible"
        } else if self.runtime.invalid {
            "invalid"
        } else if focus_visible {
            "focus-visible"
        } else if self.runtime.read_only {
            "read-only"
        } else {
            "normal"
        };
        let visible = theme
            .input_state(self.runtime.variant.token_key(), state_key)
            .expect("Vektra 默认 Input state token 必须通过测试保持有效");
        let focus_handle = self
            .focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(!self.runtime.disabled);
        let (a11y_value, a11y_runs) = self.a11y_state(window, cx);
        let weak_state = cx.weak_entity();
        #[cfg(test)]
        let caret_visible = caret_is_visible(
            editor_focused,
            self.selection.is_empty(),
            self.runtime.disabled,
            self.runtime.read_only,
        );
        #[cfg(test)]
        {
            self.last_caret_blinking = caret_should_blink(
                caret_visible,
                self.marked_range.is_some(),
                cx.reduce_motion(),
            );
        }
        let text_element = InputTextElement {
            state: cx.entity(),
            placeholder: self.runtime.placeholder.clone(),
            colors: visible,
            caret_color: resolved_caret_color(visible.caret, self.runtime.caret_color),
            caret_width: theme.input.caret_width,
            caret_opacity: if self.caret_phase_visible { 1. } else { 0. },
        };

        div()
            .id("editor")
            .debug_selector(|| "vektra-input-editor".into())
            .role(self.runtime.input_type.role())
            .track_focus(&focus_handle)
            .w_full()
            .h_full()
            .min_w_0()
            .flex()
            .items_center()
            .text_size(size.font_size)
            .line_height(size.line_height)
            .text_color(visible.foreground)
            .cursor(if self.runtime.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            })
            .when_some(self.runtime.aria_label.clone(), |this, label| {
                this.aria_label(label)
            })
            .when_some(
                self.runtime.aria_description.clone(),
                |this, description| this.aria_description(description),
            )
            .aria_value(a11y_value)
            .when(!self.runtime.placeholder.is_empty(), |this| {
                this.aria_placeholder(self.runtime.placeholder.clone())
            })
            .a11y_synthetic_children(a11y_runs)
            .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                let Some(ActionData::Value(value)) = data else {
                    return;
                };
                let _ = weak_state.update(cx, |state, cx| {
                    if state.can_edit() {
                        let len = state.value.len();
                        state.replace_user_range(0..len, value, window, cx);
                    }
                });
            })
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(text_element)
    }
}

struct InputTextElement {
    state: Entity<InputState>,
    placeholder: SharedString,
    colors: InputStateTokens,
    caret_color: Hsla,
    caret_width: Pixels,
    caret_opacity: f32,
}

struct InputTextPrepaint {
    line: ShapedLine,
    line_height: Pixels,
    selection: Option<PaintQuad>,
    caret: Option<PaintQuad>,
    display_origin: Point<Pixels>,
}

impl IntoElement for InputTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for InputTextElement {
    type RequestLayoutState = ();
    type PrepaintState = InputTextPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (display, selection, cursor, marked_range, old_scroll, focused, disabled, read_only) = {
            let state = self.state.read(cx);
            let display = state.display_text();
            (
                display.clone(),
                display.display_range(state.selection.clone()),
                display.display_offset(state.cursor_offset()),
                state
                    .marked_range
                    .clone()
                    .map(|range| display.display_range(range)),
                state.scroll_x,
                state.focus_handle.is_focused(window),
                state.runtime.disabled,
                state.runtime.read_only,
            )
        };
        let content = SharedString::from(display.text.clone());
        let is_placeholder = content.is_empty();
        let display_text = if is_placeholder {
            self.placeholder.clone()
        } else {
            content
        };
        let base_run = TextRun {
            len: display_text.len(),
            font: window.text_style().font(),
            color: if is_placeholder {
                self.colors.placeholder
            } else {
                self.colors.foreground
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = marked_text_runs(base_run, marked_range.as_ref());
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);
        let target = marked_range
            .as_ref()
            .map(|range| range.end)
            .unwrap_or(cursor);
        let scroll_content_width = line.width()
            + if selection.is_empty() {
                self.caret_width
            } else {
                Pixels::ZERO
            };
        let scroll_x = ensure_x_visible(
            old_scroll,
            line.x_for_index(target),
            scroll_content_width,
            bounds.size.width,
        );
        let scroll_x = ensure_x_visible(
            scroll_x,
            line.x_for_index(target) + self.caret_width,
            scroll_content_width,
            bounds.size.width,
        );
        let line_height = window
            .pixel_snap(window.line_height())
            .max(Pixels::ZERO)
            .min(bounds.size.height);
        let line_top = window.pixel_snap(bounds.top() + (bounds.size.height - line_height) / 2.);
        let line_bounds = Bounds::new(
            point(bounds.left(), line_top),
            size(bounds.size.width, line_height),
        );
        let display_origin = point(bounds.left() - scroll_x, line_bounds.top());
        self.state.update(cx, |state, _| {
            state.scroll_x = scroll_x;
            state.last_layout = Some(line.clone());
            state.last_display = Some(display);
            state.last_bounds = Some(bounds);
        });

        let selection_quad = (!selection.is_empty()).then(|| {
            fill(
                Bounds::from_corners(
                    point(
                        display_origin.x + line.x_for_index(selection.start),
                        line_bounds.top(),
                    ),
                    point(
                        display_origin.x + line.x_for_index(selection.end),
                        line_bounds.bottom(),
                    ),
                ),
                self.colors.selection,
            )
        });
        let caret = (selection.is_empty() && focused && !disabled && !read_only).then(|| {
            let caret_bounds = caret_bounds(
                line_bounds,
                display_origin.x + line.x_for_index(cursor),
                self.caret_width,
                line.ascent,
                line.descent,
                window.scale_factor(),
            );
            fill(caret_bounds, self.caret_color.opacity(self.caret_opacity))
        });
        #[cfg(test)]
        self.state.update(cx, |state, _| {
            state.last_caret = caret
                .as_ref()
                .map(|caret| (caret.bounds, self.caret_opacity));
        });

        InputTextPrepaint {
            line,
            line_height,
            selection: selection_quad,
            caret,
            display_origin,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.state.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.state.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = prepaint.selection.take() {
                window.paint_quad(selection);
            }
            prepaint
                .line
                .paint(
                    prepaint.display_origin,
                    prepaint.line_height,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .expect("锁定 GPUI 应能绘制已经成功 shape 的 Input 单行文本");
            if let Some(caret) = prepaint.caret.take() {
                window.paint_quad(caret);
            }
        });
    }
}

/// 单行文本输入的视觉与行为包装层。
///
/// 每个使用中的 `Input` 必须对应一个由调用方持有的 [`Entity<InputState>`]。prefix、
/// suffix 与 attached suffix 保持自己的焦点、事件、角色和无障碍子树；`disabled` 与
/// `read_only` 只约束文本编辑器和内置清除操作，调用方仍负责把相同状态传给任意交互式
/// 槽位子组件。尾部顺序为状态图标、内置 clear、suffix、分隔线、attached suffix。
#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    state: Entity<InputState>,
    placeholder: SharedString,
    variant: InputVariant,
    input_type: InputType,
    password_revealed: bool,
    size: Option<ComponentSize>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    caret_color: Option<Hsla>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    attached_suffix: Option<AnyElement>,
    clear: Option<InputClear>,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl Input {
    /// 创建绑定稳定 `ElementId` 与调用方状态的 Input。
    pub fn new(id: impl Into<ElementId>, state: Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state,
            placeholder: SharedString::default(),
            variant: InputVariant::default(),
            input_type: InputType::default(),
            password_revealed: false,
            size: None,
            disabled: false,
            read_only: false,
            invalid: false,
            caret_color: None,
            aria_label: None,
            aria_description: None,
            prefix: None,
            suffix: None,
            attached_suffix: None,
            clear: None,
            on_change: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }
    }

    /// 设置不属于实际 value 的提示文本。
    ///
    /// Placeholder 不会自动成为可访问名称。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 设置 Input 视觉变体。
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 设置单行文本输入语义。
    ///
    /// Search、Email、Phone 与 Url 只改变无障碍角色；Password 还会在未显式显示时
    /// 启用安全掩码与隐藏态剪贴板限制。该 builder 不添加图标、校验或格式化。
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// 设置受控的 Password 明文显示状态。
    ///
    /// 默认是 `false`。该值只在 [`InputType::Password`] 下生效；切换不会修改真实值、
    /// 选区、IME、撤销历史，也不会发送 [`InputEvent::Changed`]。其他输入类型会忽略
    /// 该配置。
    pub fn password_revealed(mut self, revealed: bool) -> Self {
        self.password_revealed = revealed;
        self
    }

    /// 设置 Vektra 共享语义尺寸。
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置禁用状态。
    ///
    /// 禁用后文本编辑器移出普通 Tab 顺序并拒绝输入、选区和 SetValue；任意 prefix、
    /// suffix 或 attached suffix 子组件的状态仍由调用方负责。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 设置只读状态。
    ///
    /// 只读 Input 仍可聚焦和选择；普通类型允许复制，隐藏态 Password 仍会阻止复制与
    /// 剪切。所有类型都会拒绝修改操作与 SetValue。任意槽位子组件的只读语义仍由调用方
    /// 负责。
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// 设置由外部校验逻辑驱动的 invalid 展示状态。
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// 覆盖当前 Input 实例的文本插入光标颜色。
    ///
    /// 未设置时使用当前 variant/state 的主题 `caret` token。显式颜色会覆盖包括
    /// invalid 在内的所有可编辑状态，但不会改变选区、边框、placeholder、affix 或
    /// 状态图标颜色；disabled 与 read-only 仍不会绘制插入光标。
    pub fn caret_color(mut self, color: Hsla) -> Self {
        self.caret_color = Some(color);
        self
    }

    /// 设置文本编辑器的可访问名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置文本编辑器的补充可访问描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 设置位于文本编辑 viewport 之前的任意独立元素。
    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    /// 设置位于文本编辑 viewport、状态图标与内置 clear 之后的任意独立元素。
    ///
    /// suffix 保持自己的焦点与事件；存在 clear 时，普通 Tab 顺序为 editor、clear、suffix。
    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    /// 设置与 Input 共用外壳、贴合右边缘的分段尾部操作。
    ///
    /// attached suffix 位于普通 [`Self::suffix`] 外侧，占满 Input 高度并通过主题边框色
    /// 分隔；编辑区域会优先收缩，而该区域保持自身宽度。槽位子元素继续拥有自己的焦点、
    /// 事件与状态，调用方应显式传递匹配的 `ComponentSize`、disabled 或 read-only 语义。
    pub fn attached_suffix(mut self, suffix: impl IntoElement) -> Self {
        self.attached_suffix = Some(suffix.into_any_element());
        self
    }

    /// 启用基于现有 IconButton 与 Tooltip 的内置语义清除操作。
    pub fn clearable(mut self, clear: InputClear) -> Self {
        self.clear = Some(clear);
        self
    }

    /// 注册用户值实际变化时的回调。
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的用户值变化回调。
    pub fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, SharedString, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, value: &SharedString, window, cx| {
            handler(this, value.clone(), window, cx);
        });
        self.on_change(move |value, window, cx| listener(&value, window, cx))
    }

    /// 注册 Enter 提交回调。
    pub fn on_submit(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的 Enter 提交回调。
    pub fn on_submit_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, SharedString, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, value: &SharedString, window, cx| {
            handler(this, value.clone(), window, cx);
        });
        self.on_submit(move |value, window, cx| listener(&value, window, cx))
    }

    /// 注册文本编辑器实际获得焦点时的回调。
    pub fn on_focus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_focus = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的聚焦回调。
    pub fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Focusable::on_focus_in(self, cx, handler)
    }

    /// 注册文本编辑器实际失去焦点时的回调。
    pub fn on_blur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_blur = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的失焦回调。
    pub fn on_blur_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Focusable::on_blur_in(self, cx, handler)
    }

    /// 返回稳定 ElementId。
    pub fn id(&self) -> &ElementId {
        &self.id
    }
}

impl Changeable<SharedString> for Input {
    fn on_change(self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        Input::on_change(self, handler)
    }

    fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, SharedString, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Input::on_change_in(self, cx, handler)
    }
}

impl Focusable for Input {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Input::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Input::on_blur(self, handler)
    }
}

impl Disableable for Input {
    fn disabled(self, disabled: bool) -> Self {
        Input::disabled(self, disabled)
    }
}

impl Sizable for Input {
    fn size(self, size: ComponentSize) -> Self {
        Input::size(self, size)
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Input {
            id,
            state,
            placeholder,
            variant,
            input_type,
            password_revealed,
            size: requested_size,
            disabled,
            read_only,
            invalid,
            caret_color,
            aria_label,
            aria_description,
            prefix,
            suffix,
            attached_suffix,
            clear,
            on_change,
            on_submit,
            on_focus,
            on_blur,
        } = self;
        let resolved_size = requested_size.unwrap_or_else(|| component_size(cx));
        let runtime = InputRuntime {
            id: id.clone(),
            placeholder,
            variant,
            input_type,
            password_revealed,
            size: resolved_size,
            disabled,
            read_only,
            invalid,
            caret_color,
            aria_label,
            aria_description,
            on_change,
            on_submit,
            on_focus,
            on_blur,
        };
        state.update(cx, |input_state, cx| {
            input_state.configure(runtime, window, cx);
        });

        let theme = theme::current_theme(window, cx);
        let size = theme
            .input_size(resolved_size.token_key())
            .expect("Vektra 默认 Input size token 必须通过测试保持有效");
        let states = ResolvedInputStates::new(&theme, variant);
        let base = if disabled {
            states.disabled
        } else if invalid {
            states.invalid
        } else if read_only {
            states.read_only
        } else {
            states.normal
        };
        let focus = if invalid {
            states.invalid_focused
        } else {
            states.focused
        };
        let border_width = theme.input.border_width;
        let focus_width = theme.input.focus_width;
        let attached_divider = theme.semantic.border;
        let pointer_focus_width = input_focus_border_width(false, border_width, focus_width);
        let keyboard_focus_width = input_focus_border_width(true, border_width, focus_width);
        let has_attached_suffix = attached_suffix.is_some();
        let focus_handle = state.read(cx).focus_handle.clone();
        let clear_view = clear.map(|clear| {
            let clear_state = state.clone();
            let initial_clear = clear.clone();
            let view = window.use_keyed_state((id.clone(), "input-clear"), cx, move |_, cx| {
                InputClearView::new(clear_state, initial_clear, cx)
            });
            view.update(cx, |view, _| {
                view.clear = clear;
            });
            view
        });

        let content = div()
            .debug_selector(|| "vektra-input-content".into())
            .flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .items_center()
            .gap(size.gap)
            .px(size.padding_x)
            .when_some(prefix, |this, prefix| this.child(render_slot(prefix, size)))
            .child(div().flex_1().min_w_0().h_full().child(state))
            .when(invalid, |this| {
                this.child(
                    div()
                        .debug_selector(|| "vektra-input-invalid".into())
                        .flex()
                        .flex_none()
                        .child(
                            Icon::new(IconSource::asset("components/input/invalid.svg"))
                                .size(size.status_size)
                                .color(base.status),
                        ),
                )
            })
            .when_some(clear_view, |this, clear| this.child(clear))
            .when_some(suffix, |this, suffix| this.child(render_slot(suffix, size)));

        div()
            .id(id)
            .debug_selector(|| "vektra-input".into())
            .track_focus(&focus_handle)
            .w_full()
            .h(size.height)
            .min_h(size.height)
            .min_w_0()
            .flex()
            .items_center()
            .rounded(input_radius(variant, size.radius))
            .bg(base.background)
            .text_color(base.affix)
            .when(has_attached_suffix, |this| this.overflow_hidden())
            .when(variant != InputVariant::Underline, |this| {
                this.border(border_width).border_color(base.border)
            })
            .when(variant == InputVariant::Underline, |this| {
                this.border_b(border_width).border_color(base.border)
            })
            .when(!disabled && !read_only && !invalid, |this| {
                this.hover(move |style| {
                    let style = style
                        .bg(states.hover.background)
                        .text_color(states.hover.affix);
                    if variant == InputVariant::Borderless {
                        style
                    } else {
                        style.border_color(states.hover.border)
                    }
                })
            })
            .when(!disabled && variant == InputVariant::Borderless, |this| {
                this.focus(move |style| {
                    style
                        .border(pointer_focus_width)
                        .border_color(focus.border)
                        .bg(focus.background)
                        .text_color(focus.affix)
                })
            })
            .when(!disabled, |this| {
                this.focus_visible(move |style| {
                    let style = style
                        .bg(focus.background)
                        .border_color(focus.border)
                        .text_color(focus.affix);
                    if variant == InputVariant::Underline {
                        style.border_b(keyboard_focus_width)
                    } else {
                        style.border(keyboard_focus_width)
                    }
                })
            })
            .child(content)
            .when_some(attached_suffix, |this, suffix| {
                this.child(render_attached_suffix(
                    suffix,
                    border_width,
                    attached_divider,
                ))
            })
    }
}

fn render_slot(slot: AnyElement, size: InputSizeTokens) -> gpui::Div {
    div()
        .flex()
        .flex_shrink_1()
        .min_w_0()
        .min_h(size.slot_size)
        .items_center()
        .child(slot)
}

fn render_attached_suffix(
    slot: AnyElement,
    divider_width: Pixels,
    divider_color: Hsla,
) -> gpui::Div {
    div()
        .debug_selector(|| "vektra-input-attached-suffix".into())
        .flex()
        .flex_none()
        .h_full()
        .items_center()
        .border_l(divider_width)
        .border_color(divider_color)
        .child(slot)
}

struct InputClearView {
    state: Entity<InputState>,
    clear: InputClear,
    _state_subscription: Subscription,
}

impl InputClearView {
    fn new(state: Entity<InputState>, clear: InputClear, cx: &mut Context<Self>) -> Self {
        let state_subscription = cx.observe(&state, |_, _, cx| cx.notify());
        Self {
            state,
            clear,
            _state_subscription: state_subscription,
        }
    }
}

impl Render for InputClearView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = {
            let state = self.state.read(cx);
            !state.value.is_empty() && !state.runtime.disabled && !state.runtime.read_only
        };
        let weak_state = self.state.downgrade();
        let mut button = IconButton::new("clear", IconSource::asset("components/input/clear.svg"))
            .variant(IconButtonVariant::Ghost)
            .size(ComponentSize::Xs)
            .aria_label(self.clear.aria_label.clone())
            .tooltip_placement(self.clear.tooltip_placement)
            .on_click(move |_, window, cx| {
                let Some(state) = weak_state.upgrade() else {
                    return;
                };
                let focus_handle = state.read(cx).focus_handle.clone();
                state.update(cx, |state, cx| state.user_clear(window, cx));
                window.focus(&focus_handle, cx);
            });
        if let Some(tooltip) = self.clear.tooltip.clone() {
            button = button.tooltip(tooltip);
        }
        div().when(visible, |this| {
            this.child(
                div()
                    .debug_selector(|| "vektra-input-clear".into())
                    .flex()
                    .flex_none()
                    .child(button),
            )
        })
    }
}

#[derive(Clone, Copy)]
struct ResolvedInputStates {
    normal: InputStateTokens,
    hover: InputStateTokens,
    focused: InputStateTokens,
    invalid: InputStateTokens,
    invalid_focused: InputStateTokens,
    read_only: InputStateTokens,
    disabled: InputStateTokens,
}

impl ResolvedInputStates {
    fn new(theme: &ResolvedTheme, variant: InputVariant) -> Self {
        let variant = variant.token_key();
        let state = |key| {
            theme
                .input_state(variant, key)
                .expect("Vektra 默认 Input state token 必须通过测试保持有效")
        };
        Self {
            normal: state("normal"),
            hover: state("hover"),
            focused: state("focus-visible"),
            invalid: state("invalid"),
            invalid_focused: state("invalid-focus-visible"),
            read_only: state("read-only"),
            disabled: state("disabled"),
        }
    }
}

fn caret_is_visible(focused: bool, selection_empty: bool, disabled: bool, read_only: bool) -> bool {
    focused && selection_empty && !disabled && !read_only
}

fn caret_should_blink(visible: bool, composing: bool, reduce_motion: bool) -> bool {
    visible && !composing && !reduce_motion
}

fn resolved_caret_color(theme_color: Hsla, explicit_color: Option<Hsla>) -> Hsla {
    explicit_color.unwrap_or(theme_color)
}

fn input_radius(variant: InputVariant, radius: Pixels) -> Pixels {
    if variant == InputVariant::Underline {
        Pixels::ZERO
    } else {
        radius
    }
}

fn input_focus_border_width(
    focus_visible: bool,
    border_width: Pixels,
    focus_width: Pixels,
) -> Pixels {
    if focus_visible {
        focus_width
    } else {
        border_width
    }
}

fn snap_to_device_pixel(value: Pixels, scale_factor: f32) -> Pixels {
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0. {
        scale_factor
    } else {
        1.
    };
    px((value.as_f32() * scale_factor).round() / scale_factor)
}

fn caret_bounds(
    line_bounds: Bounds<Pixels>,
    x: Pixels,
    width: Pixels,
    ascent: Pixels,
    descent: Pixels,
    scale_factor: f32,
) -> Bounds<Pixels> {
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0. {
        scale_factor
    } else {
        1.
    };
    let device_pixel = px(1. / scale_factor);
    let width = if width > Pixels::ZERO {
        snap_to_device_pixel(width, scale_factor).max(device_pixel)
    } else {
        Pixels::ZERO
    };
    let height = snap_to_device_pixel(
        (ascent + descent)
            .max(device_pixel)
            .min(line_bounds.size.height),
        scale_factor,
    )
    .min(line_bounds.size.height);
    let centered_y = line_bounds.top() + (line_bounds.size.height - height) / 2.;
    let y = snap_to_device_pixel(centered_y, scale_factor)
        .max(line_bounds.top())
        .min(line_bounds.bottom() - height);
    Bounds::new(
        point(snap_to_device_pixel(x, scale_factor), y),
        size(width, height),
    )
}

fn stop_key_event(window: &mut Window, cx: &mut Context<InputState>) {
    window.prevent_default();
    cx.stop_propagation();
}

fn plain_or_shift_modifiers(modifiers: Modifiers) -> bool {
    !modifiers.control && !modifiers.alt && !modifiers.platform && !modifiers.function
}

fn secondary_shortcut_modifiers(modifiers: Modifiers, shift: bool) -> bool {
    modifiers.secondary()
        && modifiers.shift == shift
        && modifiers.number_of_modifiers() == if shift { 2 } else { 1 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HorizontalMovement {
    Grapheme,
    Word,
    #[cfg(target_os = "macos")]
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Deletion {
    Grapheme,
    Word,
    #[cfg(target_os = "macos")]
    Line,
}

#[cfg(target_os = "macos")]
fn horizontal_movement_kind(modifiers: Modifiers) -> Option<HorizontalMovement> {
    if plain_or_shift_modifiers(modifiers) {
        Some(HorizontalMovement::Grapheme)
    } else if modifiers.alt && !modifiers.control && !modifiers.platform && !modifiers.function {
        Some(HorizontalMovement::Word)
    } else if (modifiers.platform ^ modifiers.function) && !modifiers.control && !modifiers.alt {
        Some(HorizontalMovement::Line)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn horizontal_movement_kind(modifiers: Modifiers) -> Option<HorizontalMovement> {
    if plain_or_shift_modifiers(modifiers) {
        Some(HorizontalMovement::Grapheme)
    } else if modifiers.control && !modifiers.alt && !modifiers.platform && !modifiers.function {
        Some(HorizontalMovement::Word)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn deletion_kind(modifiers: Modifiers) -> Option<Deletion> {
    if modifiers.shift {
        None
    } else if plain_or_shift_modifiers(modifiers) {
        Some(Deletion::Grapheme)
    } else if modifiers.alt && !modifiers.control && !modifiers.platform && !modifiers.function {
        Some(Deletion::Word)
    } else if modifiers.platform && !modifiers.control && !modifiers.alt && !modifiers.function {
        Some(Deletion::Line)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn deletion_kind(modifiers: Modifiers) -> Option<Deletion> {
    if modifiers.shift {
        None
    } else if plain_or_shift_modifiers(modifiers) {
        Some(Deletion::Grapheme)
    } else if modifiers.control && !modifiers.alt && !modifiers.platform && !modifiers.function {
        Some(Deletion::Word)
    } else {
        None
    }
}

fn horizontal_target(state: &InputState, right: bool, kind: HorizontalMovement) -> usize {
    match kind {
        HorizontalMovement::Grapheme if right => state.next_boundary(state.cursor_offset()),
        HorizontalMovement::Grapheme => state.previous_boundary(state.cursor_offset()),
        HorizontalMovement::Word if right => state.next_word_boundary(state.cursor_offset()),
        HorizontalMovement::Word => state.previous_word_boundary(state.cursor_offset()),
        #[cfg(target_os = "macos")]
        HorizontalMovement::Line if right => state.value.len(),
        #[cfg(target_os = "macos")]
        HorizontalMovement::Line => 0,
    }
}

fn normalize_single_line(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                output.push(' ');
            }
            '\n' => output.push(' '),
            _ => output.push(ch),
        }
    }
    output
}

fn utf16_to_utf8(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (byte, ch) in text.char_indices() {
        if utf16 >= offset {
            return byte;
        }
        let next = utf16 + ch.len_utf16();
        if offset < next {
            return byte;
        }
        utf16 = next;
    }
    text.len()
}

fn utf8_to_utf16(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    text.char_indices()
        .take_while(|(byte, _)| *byte < offset)
        .map(|(_, ch)| ch.len_utf16())
        .sum()
}

fn range_from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    normalize_selection(
        text,
        utf16_to_utf8(text, range.start)..utf16_to_utf8(text, range.end),
    )
}

fn range_to_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    utf8_to_utf16(text, range.start)..utf8_to_utf16(text, range.end)
}

fn previous_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
        .unwrap_or(0)
}

fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .find(|index| *index > offset)
        .unwrap_or(text.len())
}

fn nearest_paired_boundary(offset: usize, from: &[usize], to: &[usize]) -> usize {
    debug_assert_eq!(from.len(), to.len());
    debug_assert!(!from.is_empty());
    let index = match from.binary_search(&offset) {
        Ok(index) => index,
        Err(0) => 0,
        Err(index) if index == from.len() => from.len() - 1,
        Err(index) => {
            let before = index - 1;
            if offset - from[before] <= from[index] - offset {
                before
            } else {
                index
            }
        }
    };
    to[index]
}

fn previous_word_boundary(text: &str, offset: usize) -> usize {
    text.split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, _)| start)
        .take_while(|start| *start < offset)
        .last()
        .unwrap_or(0)
}

fn next_word_boundary(text: &str, offset: usize) -> usize {
    text.split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, segment)| start + segment.len())
        .find(|end| *end > offset)
        .unwrap_or(text.len())
}

fn nearest_grapheme_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    if offset == text.len()
        || text
            .grapheme_indices(true)
            .any(|(index, _)| index == offset)
    {
        return offset;
    }
    let before = previous_grapheme_boundary(text, offset + 1);
    let after = next_grapheme_boundary(text, before);
    if offset - before <= after.saturating_sub(offset) {
        before
    } else {
        after
    }
}

fn normalize_selection(text: &str, range: Range<usize>) -> Range<usize> {
    let start = nearest_grapheme_boundary(text, range.start.min(text.len()));
    let end = nearest_grapheme_boundary(text, range.end.min(text.len()));
    start.min(end)..start.max(end)
}

fn word_range_at(text: &str, offset: usize) -> Range<usize> {
    if text.is_empty() {
        return 0..0;
    }
    let offset = offset.min(text.len().saturating_sub(1));
    if let Some((start, word)) = text
        .unicode_word_indices()
        .find(|(start, word)| offset >= *start && offset < *start + word.len())
    {
        return start..start + word.len();
    }
    let start = nearest_grapheme_boundary(text, offset);
    start..next_grapheme_boundary(text, start)
}

fn ensure_x_visible(
    scroll_x: Pixels,
    target_x: Pixels,
    content_width: Pixels,
    viewport_width: Pixels,
) -> Pixels {
    let viewport_width = viewport_width.max(Pixels::ZERO);
    let max_scroll = (content_width - viewport_width).max(Pixels::ZERO);
    let mut scroll_x = scroll_x.max(Pixels::ZERO).min(max_scroll);
    if target_x < scroll_x {
        scroll_x = target_x;
    } else if target_x > scroll_x + viewport_width {
        scroll_x = target_x - viewport_width;
    }
    scroll_x.max(Pixels::ZERO).min(max_scroll)
}

fn marked_text_runs(base: TextRun, marked_range: Option<&Range<usize>>) -> Vec<TextRun> {
    let Some(marked_range) = marked_range else {
        return vec![base];
    };
    [
        TextRun {
            len: marked_range.start,
            ..base.clone()
        },
        TextRun {
            len: marked_range.end.saturating_sub(marked_range.start),
            underline: Some(UnderlineStyle {
                color: Some(base.color),
                thickness: px(1.),
                wavy: false,
            }),
            ..base.clone()
        },
        TextRun {
            len: base.len.saturating_sub(marked_range.end),
            ..base
        },
    ]
    .into_iter()
    .filter(|run| run.len > 0)
    .collect()
}

fn char_index_for_byte(text: &str, byte_offset: usize) -> usize {
    text.char_indices()
        .take_while(|(byte, _)| *byte < byte_offset.min(text.len()))
        .count()
}

fn a11y_text_position(
    char_index: usize,
    synthetic_node_id: impl Fn(u64) -> accesskit::NodeId,
) -> accesskit::TextPosition {
    let chunk_index = if char_index > 0 && char_index.is_multiple_of(MAX_CHARS_PER_TEXT_RUN) {
        char_index / MAX_CHARS_PER_TEXT_RUN - 1
    } else {
        char_index / MAX_CHARS_PER_TEXT_RUN
    };
    accesskit::TextPosition {
        node: synthetic_node_id(chunk_index as u64),
        character_index: char_index - chunk_index * MAX_CHARS_PER_TEXT_RUN,
    }
}

fn build_a11y_text_runs(
    text: &str,
    selection_tail: usize,
    selection_head: usize,
    synthetic_node_id: impl Fn(u64) -> accesskit::NodeId,
) -> (
    Vec<(accesskit::NodeId, accesskit::Node)>,
    accesskit::TextSelection,
) {
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();
    let num_chunks = total_chars.div_ceil(MAX_CHARS_PER_TEXT_RUN).max(1);
    let mut word_starts = Vec::new();
    let mut was_word = false;
    for (index, ch) in chars.iter().enumerate() {
        let is_word = ch.is_alphanumeric() || *ch == '_';
        if is_word && !was_word {
            word_starts.push(index);
        }
        was_word = is_word;
    }

    let mut runs = Vec::with_capacity(num_chunks);
    for chunk_index in 0..num_chunks {
        let char_start = chunk_index * MAX_CHARS_PER_TEXT_RUN;
        let char_end = (char_start + MAX_CHARS_PER_TEXT_RUN).min(total_chars);
        let chunk = &chars[char_start..char_end];
        let mut node = accesskit::Node::new(accesskit::Role::TextRun);
        node.set_text_direction(accesskit::TextDirection::LeftToRight);
        node.set_value(chunk.iter().collect::<String>());
        node.set_character_lengths(
            chunk
                .iter()
                .map(|ch| ch.len_utf8() as u8)
                .collect::<Vec<_>>(),
        );
        node.set_word_starts(
            word_starts
                .iter()
                .filter(|start| **start >= char_start && **start < char_end)
                .map(|start| (*start - char_start) as u8)
                .collect::<Vec<_>>(),
        );
        if chunk_index > 0 {
            node.set_previous_on_line(synthetic_node_id(chunk_index as u64 - 1));
        }
        if chunk_index + 1 < num_chunks {
            node.set_next_on_line(synthetic_node_id(chunk_index as u64 + 1));
        }
        runs.push((synthetic_node_id(chunk_index as u64), node));
    }
    let anchor = a11y_text_position(
        char_index_for_byte(text, selection_tail),
        &synthetic_node_id,
    );
    let focus = a11y_text_position(
        char_index_for_byte(text, selection_head),
        &synthetic_node_id,
    );
    (runs, accesskit::TextSelection { anchor, focus })
}

fn push_a11y_text_runs(
    builder: &mut A11ySubtreeBuilder,
    text: &str,
    selection_tail: usize,
    selection_head: usize,
) {
    let (runs, selection) = build_a11y_text_runs(text, selection_tail, selection_head, |chunk| {
        builder.synthetic_node_id(chunk)
    });
    for (id, node) in runs {
        builder.push_child(id, node);
    }
    builder.parent_node().set_text_selection(selection);
}

#[cfg(test)]
#[path = "../tests/unit/input.rs"]
mod tests;
