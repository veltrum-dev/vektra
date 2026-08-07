//! Switch 组件。

use crate::{
    button::{self, ClickHandler},
    focus::{self, FocusHandler},
    icon::{Icon, IconSource, IntoIconSource},
    size::{ComponentSize, component_size},
    theme,
    traits::{Changeable, Clickable, Disableable, Focusable, Sizable},
};
use gpui::{
    Animation, AnimationExt, AnyElement, App, ClickEvent, Context, CursorStyle, ElementId,
    InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, Modifiers, MouseButton,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Toggled, Transformation, Window, div, percentage, prelude::FluentBuilder, px, svg,
};
use std::{rc::Rc, time::Duration};
use vektra_theme::{ResolvedTheme, SwitchSizeTokens, SwitchStateTokens};

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
enum ActivationHandler {
    Change(ChangeHandler),
    Click(ClickHandler),
}

pub(crate) const DEFAULT_SWITCH_TRANSITION_DURATION: Duration = Duration::from_millis(180);
const SWITCH_LOADING_SPINNER_DURATION: Duration = Duration::from_millis(900);

/// Switch 轨道内的受限状态内容。
///
/// 状态内容只用于补充当前开启或关闭状态的视觉提示，不创建可访问节点或交互目标，
/// 也不替代 [`Switch::label`] 或 [`Switch::aria_label`] 提供的可访问名称。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchContent {
    kind: SwitchContentKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SwitchContentKind {
    Text(SharedString),
    Icon(IconSource),
    IconText {
        icon: IconSource,
        text: SharedString,
    },
}

impl SwitchContent {
    /// 创建纯文字状态内容。
    ///
    /// 文字始终保持单行，并按主题提供的最大宽度截断，适合“开启”“关闭”等短文本。
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self {
            kind: SwitchContentKind::Text(text.into()),
        }
    }

    /// 创建纯装饰图标状态内容。
    ///
    /// 图标复用 Vektra 的 [`IntoIconSource`] 契约，不会创建额外可访问节点。
    pub fn icon(icon: impl IntoIconSource) -> Self {
        Self {
            kind: SwitchContentKind::Icon(icon.into_icon_source()),
        }
    }

    /// 创建图标在前、短文字在后的状态内容。
    ///
    /// 图标和文字按逻辑方向排列；文字保持单行并按主题上限截断。
    pub fn icon_text(icon: impl IntoIconSource, text: impl Into<SharedString>) -> Self {
        Self {
            kind: SwitchContentKind::IconText {
                icon: icon.into_icon_source(),
                text: text.into(),
            },
        }
    }
}

/// Vektra Switch。
///
/// Switch 用于表达立即生效的开启/关闭设置。它是受控 builder 组件，不在内部保存
/// 业务状态；调用方应在每次 render 时通过 [`Self::checked`] 提供当前值，并在
/// [`Self::on_change`] 或 [`Self::on_change_in`] 中更新宿主状态。
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    loading: bool,
    transition_duration: Duration,
    label: Option<SharedString>,
    size: Option<ComponentSize>,
    cursor_style: Option<CursorStyle>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    checked_content: Option<SwitchContent>,
    unchecked_content: Option<SwitchContent>,
    on_activate: Option<ActivationHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl Switch {
    /// 创建一个带稳定 `ElementId` 的 Switch。
    ///
    /// 新建后默认关闭且可用，没有可见 label、显式尺寸或回调。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            loading: false,
            transition_duration: DEFAULT_SWITCH_TRANSITION_DURATION,
            label: None,
            size: None,
            cursor_style: None,
            aria_label: None,
            aria_description: None,
            checked_content: None,
            unchecked_content: None,
            on_activate: None,
            on_focus: None,
            on_blur: None,
        }
    }

    /// 设置当前受控开启状态。
    ///
    /// 该值不是初始值；回调不会自动修改它，宿主状态变化后应在下一次 render 中继续
    /// 传入最新值。
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// 设置 disabled 状态。
    ///
    /// disabled 时不进入正常 Tab 顺序，鼠标和 Space 都不会触发状态变化。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 设置受控 loading 状态。
    ///
    /// loading 时 thumb 保持当前受控位置并显示不确定进度指示器；Switch 保留焦点和
    /// Tab 停靠，但会消费鼠标、Enter 与 Space 事件且不触发状态变化。异步任务、错误
    /// 处理和 checked 的乐观或悲观更新均由宿主应用负责。
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// 设置 checked 状态切换的动画时长。
    ///
    /// 该时长同时控制 thumb 位移和状态内容交叉淡化；默认是 180ms，固定使用
    /// ease-out cubic。传入 [`Duration::ZERO`] 会让下一次切换直接收敛到终态。
    /// reduced-motion 始终优先，且该设置不会影响 loading spinner 的循环速度。
    /// 通常建议使用 100–400ms。
    pub fn transition_duration(mut self, duration: Duration) -> Self {
        self.transition_duration = duration;
        self
    }

    /// 设置紧随 track 之后的可见文本 label。
    ///
    /// 可见 label 与 track 共享同一交互区域，并默认作为可访问名称。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置组件级显式语义尺寸。
    ///
    /// 未调用时，在渲染阶段读取全局 [`ComponentSize`]。
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置可用状态下的鼠标光标。
    ///
    /// disabled 状态始终显示不可操作光标，不会被此设置绕过。
    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = Some(cursor_style);
        self
    }

    /// 设置辅助技术朗读的名称。
    ///
    /// 该名称会覆盖可见 label；没有可见 label 时调用方应提供该名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置辅助技术朗读的补充描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 设置开启状态显示在 thumb 腾出的逻辑起始区域中的内容。
    ///
    /// 重复调用时最后一次设置生效；只配置一侧时另一侧保持为空。
    pub fn checked_content(mut self, content: SwitchContent) -> Self {
        self.checked_content = Some(content);
        self
    }

    /// 设置关闭状态显示在 thumb 腾出的逻辑末端区域中的内容。
    ///
    /// 重复调用时最后一次设置生效；只配置一侧时另一侧保持为空。
    pub fn unchecked_content(mut self, content: SwitchContent) -> Self {
        self.unchecked_content = Some(content);
        self
    }

    /// 注册受控状态变化回调。
    ///
    /// 每次有效激活只同步调用一次，参数为 `!checked`。这不是运行时事件总线，宿主
    /// 必须自行更新业务状态并触发重绘。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(ActivationHandler::Change(Rc::new(handler)));
        self
    }

    /// 注册标准原始激活回调。
    ///
    /// 该入口适合在宿主中先启动后台请求，并在请求成功后再更新受控 checked。Switch
    /// 不会因回调执行而自行改变状态。`on_click` 与 [`Self::on_change`] 共享同一个激活
    /// handler 槽，连续调用时后调用者生效，因此一次激活不会重复触发两套回调。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(ActivationHandler::Click(Rc::new(handler)));
        self
    }

    /// 注册绑定宿主 Entity 的标准原始激活回调。
    ///
    /// handler 可以读取宿主当前状态、启动异步请求，并在请求完成后更新 checked。
    /// 与 [`Self::on_change_in`] 连续调用时后调用者生效。
    pub fn on_click_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &ClickEvent, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        <Self as Clickable>::on_click_in(self, cx, handler)
    }

    /// 注册绑定宿主 Entity 的受控状态变化回调。
    ///
    /// `_in` 表示通过 [`Context::listener`] 绑定 Entity；Entity 销毁后保留 GPUI
    /// listener 的弱引用/no-op 生命周期语义。
    pub fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, bool, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, next_checked: &bool, window, cx| {
            handler(this, *next_checked, window, cx);
        });
        self.on_change(move |next_checked, window, cx| listener(&next_checked, window, cx))
    }

    /// 注册组件实际获得焦点时调用的回调。
    ///
    /// checked 或其他 builder 状态变化不会触发该回调。
    pub fn on_focus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_focus = Some(Rc::new(handler));
        self
    }

    /// 注册绑定宿主 Entity 的真实聚焦回调。
    pub fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Focusable::on_focus_in(self, cx, handler)
    }

    /// 注册组件实际失去焦点时调用的回调。
    ///
    /// 焦点生命周期与 checked 状态变化彼此独立。
    pub fn on_blur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_blur = Some(Rc::new(handler));
        self
    }

    /// 注册绑定宿主 Entity 的真实失焦回调。
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

    pub(crate) fn resolved_size(&self, cx: &App) -> ComponentSize {
        self.size.unwrap_or_else(|| component_size(cx))
    }

    fn accessible_label(&self) -> Option<SharedString> {
        self.aria_label.clone().or_else(|| self.label.clone())
    }

    #[cfg(test)]
    pub(crate) fn is_checked(&self) -> bool {
        self.checked
    }
    #[cfg(test)]
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }
    #[cfg(test)]
    pub(crate) fn is_loading(&self) -> bool {
        self.loading
    }
    #[cfg(test)]
    pub(crate) fn transition_duration_value(&self) -> Duration {
        self.transition_duration
    }
    #[cfg(test)]
    pub(crate) fn label_text(&self) -> Option<&SharedString> {
        self.label.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn aria_label_text(&self) -> Option<&SharedString> {
        self.aria_label.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn aria_description_text(&self) -> Option<&SharedString> {
        self.aria_description.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn explicit_size(&self) -> Option<ComponentSize> {
        self.size
    }
    #[cfg(test)]
    pub(crate) fn cursor_style_value(&self) -> Option<CursorStyle> {
        self.cursor_style
    }
    #[cfg(test)]
    pub(crate) fn checked_content_value(&self) -> Option<&SwitchContent> {
        self.checked_content.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn unchecked_content_value(&self) -> Option<&SwitchContent> {
        self.unchecked_content.as_ref()
    }
    #[cfg(test)]
    fn loading_indicator_id(&self) -> ElementId {
        (self.id.clone(), "loading-indicator").into()
    }
    #[cfg(test)]
    fn loading_animation_id(&self) -> ElementId {
        (self.id.clone(), "loading-animation").into()
    }
}

impl Disableable for Switch {
    fn disabled(self, disabled: bool) -> Self {
        Switch::disabled(self, disabled)
    }
}

impl Changeable<bool> for Switch {
    fn on_change(self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        Switch::on_change(self, handler)
    }

    fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, bool, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Switch::on_change_in(self, cx, handler)
    }
}

impl Clickable for Switch {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        Switch::on_click(self, handler)
    }

    fn cursor_style(self, cursor_style: CursorStyle) -> Self {
        Switch::cursor_style(self, cursor_style)
    }
}

impl Focusable for Switch {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Switch::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Switch::on_blur(self, handler)
    }
}

impl Sizable for Switch {
    fn size(self, size: ComponentSize) -> Self {
        Switch::size(self, size)
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_state = focus::state_for(
            &self.id,
            !self.disabled,
            self.on_focus.clone(),
            self.on_blur.clone(),
            window,
            cx,
        );
        let theme = theme::current_theme(window, cx);
        let accessible_label = self.accessible_label();
        let size = theme
            .switch_size(self.resolved_size(cx).token_key())
            .expect("Vektra 默认 Switch size token 必须通过测试保持有效");
        let states = ResolvedSwitchStates::new(&theme, self.checked);
        let motion = motion_for(&self.id, self.checked, self.transition_duration, window, cx);
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };
        let on_activate = self
            .on_activate
            .clone()
            .filter(|_| !self.disabled && !self.loading);
        let on_click: Option<ClickHandler> = on_activate.map(|handler| {
            let next_checked = !self.checked;
            Rc::new(
                move |event: &ClickEvent, window: &mut Window, cx: &mut App| match &handler {
                    ActivationHandler::Change(handler) => handler(next_checked, window, cx),
                    ActivationHandler::Click(handler) => handler(event, window, cx),
                },
            ) as ClickHandler
        });
        let interaction_group: SharedString = format!("vektra-switch-{:?}", self.id).into();
        let track_id: ElementId = (self.id.clone(), "track").into();
        let thumb_id: ElementId = (self.id.clone(), "thumb").into();
        let loading_indicator_id: ElementId = (self.id.clone(), "loading-indicator").into();
        let loading_animation_id: ElementId = (self.id.clone(), "loading-animation").into();
        let has_content = self.checked_content.is_some() || self.unchecked_content.is_some();
        let content_width = stable_content_width(
            self.checked_content.as_ref(),
            self.unchecked_content.as_ref(),
            size,
        );
        let track_width =
            track_width_for(has_content, content_width, size, theme.switch.border_width);
        let track_height = if has_content {
            size.content_track_height
        } else {
            size.track_height
        };
        let track_padding = if has_content {
            size.content_track_padding
        } else {
            size.track_padding
        };
        let thumb_size = if has_content {
            size.content_thumb_size
        } else {
            size.thumb_size
        };
        let track_radius = if has_content {
            track_height / 2.
        } else {
            size.track_radius
        };
        let thumb_radius = if has_content {
            thumb_size / 2.
        } else {
            size.thumb_radius
        };

        let thumb_start = thumb_offset(
            motion.from_checked,
            has_content,
            content_width,
            size,
            theme.switch.border_width,
        );
        let thumb_target = thumb_offset(
            self.checked,
            has_content,
            content_width,
            size,
            theme.switch.border_width,
        );
        let thumb = div()
            .id(thumb_id)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .size(thumb_size)
            .rounded(thumb_radius)
            .bg(visible.thumb)
            .when(!self.disabled && !self.loading, |this| {
                this.group_hover(interaction_group.clone(), move |style| {
                    style.bg(states.hover.thumb)
                })
                .group_active(interaction_group.clone(), move |style| {
                    style.bg(states.pressed.thumb)
                })
            })
            .when(self.loading, |this| {
                this.child(render_loading_indicator(
                    loading_indicator_id,
                    loading_animation_id,
                    accessible_label.clone(),
                    size.spinner_size,
                    visible.spinner,
                ))
            });
        let thumb: AnyElement = if motion.animate {
            thumb
                .with_animation(
                    ElementId::named_usize("thumb-motion", motion.generation),
                    Animation::new(motion.duration).with_easing(switch_ease_out_cubic),
                    move |this, delta| {
                        this.ml(interpolate_pixels(thumb_start, thumb_target, delta))
                    },
                )
                .into_any_element()
        } else {
            thumb.ml(thumb_target).into_any_element()
        };

        let content_layer = has_content.then(|| {
            render_content_layer(
                self.checked_content,
                self.unchecked_content,
                ContentRenderStyle {
                    size,
                    content_width,
                    motion,
                    checked: self.checked,
                },
            )
        });

        let track = div()
            .id(track_id)
            .flex()
            .flex_none()
            .relative()
            .overflow_hidden()
            .items_center()
            .justify_start()
            .w(track_width)
            .h(track_height)
            .p(track_padding)
            .rounded(track_radius)
            .border(theme.switch.border_width)
            .border_color(visible.track_border)
            .bg(visible.track_background)
            .when(!self.disabled && !self.loading, |this| {
                this.group_hover(interaction_group.clone(), move |style| {
                    style
                        .bg(states.hover.track_background)
                        .border_color(states.hover.track_border)
                        .text_color(states.hover.content)
                })
                .group_active(interaction_group.clone(), move |style| {
                    style
                        .bg(states.pressed.track_background)
                        .border_color(states.pressed.track_border)
                        .text_color(states.pressed.content)
                })
            })
            .when_some(content_layer, |this, content| this.child(content))
            .child(thumb);

        let element = div()
            .id(self.id.clone())
            .group(interaction_group)
            .debug_selector(|| "vektra-switch".into())
            .role(Role::Switch)
            .aria_toggled(toggled_state(self.checked))
            .when_some(accessible_label, |this, label| this.aria_label(label))
            .when_some(self.aria_description, |this, description| {
                this.aria_description(description)
            })
            .flex()
            .items_center()
            .gap(size.label_gap)
            .min_h(size.hit_size)
            .min_w(size.hit_size)
            .py(size.hit_padding_y)
            .pr(size.hit_padding_x)
            .text_size(size.font_size)
            .line_height(size.line_height)
            .text_color(visible.content)
            .child(track)
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .min_w_0()
                        .whitespace_normal()
                        .text_color(visible.label)
                        .child(label),
                )
            });

        let element = apply_interaction(
            element,
            SwitchInteraction {
                disabled: self.disabled,
                busy: self.loading,
                on_click,
                cursor_style: self.cursor_style,
                hover: states.hover,
                pressed: states.pressed,
                focused: states.focused,
                focus_width: theme.switch.focus_width,
            },
        );
        focus::attach_interaction(element, &focus_state, !self.disabled, cx)
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSwitchStates {
    normal: SwitchStateTokens,
    hover: SwitchStateTokens,
    pressed: SwitchStateTokens,
    focused: SwitchStateTokens,
    disabled: SwitchStateTokens,
}

impl ResolvedSwitchStates {
    fn new(theme: &ResolvedTheme, checked: bool) -> Self {
        let visual_state = if checked { "checked" } else { "unchecked" };
        Self {
            normal: theme
                .switch_state(visual_state, "normal")
                .expect("Vektra 默认 Switch normal token 必须通过测试保持有效"),
            hover: theme
                .switch_state(visual_state, "hover")
                .expect("Vektra 默认 Switch hover token 必须通过测试保持有效"),
            pressed: theme
                .switch_state(visual_state, "pressed")
                .expect("Vektra 默认 Switch pressed token 必须通过测试保持有效"),
            focused: theme
                .switch_state(visual_state, "focus-visible")
                .expect("Vektra 默认 Switch focus token 必须通过测试保持有效"),
            disabled: theme
                .switch_state(visual_state, "disabled")
                .expect("Vektra 默认 Switch disabled token 必须通过测试保持有效"),
        }
    }
}

struct SwitchInteraction {
    disabled: bool,
    busy: bool,
    on_click: Option<ClickHandler>,
    cursor_style: Option<CursorStyle>,
    hover: SwitchStateTokens,
    pressed: SwitchStateTokens,
    focused: SwitchStateTokens,
    focus_width: gpui::Pixels,
}

struct SwitchMotionState {
    checked: bool,
    from_checked: bool,
    generation: usize,
    duration: Duration,
    animate_generation: Option<usize>,
}

#[derive(Clone, Copy)]
struct SwitchMotion {
    from_checked: bool,
    generation: usize,
    duration: Duration,
    animate: bool,
}

impl SwitchMotionState {
    const fn new(checked: bool, duration: Duration) -> Self {
        Self {
            checked,
            from_checked: checked,
            generation: 0,
            duration,
            animate_generation: None,
        }
    }

    fn update(&mut self, checked: bool, duration: Duration, reduce_motion: bool) {
        if self.checked == checked {
            return;
        }

        self.from_checked = self.checked;
        self.checked = checked;
        self.generation = self.generation.wrapping_add(1);
        self.duration = duration;
        self.animate_generation =
            (!duration.is_zero() && !reduce_motion).then_some(self.generation);
    }
}

fn motion_for(
    id: &ElementId,
    checked: bool,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> SwitchMotion {
    let state = window.use_keyed_state((id.clone(), "motion"), cx, move |_, _| {
        SwitchMotionState::new(checked, duration)
    });
    let reduce_motion = cx.reduce_motion();
    state.update(cx, |state, _| {
        state.update(checked, duration, reduce_motion);
    });
    let state = state.read(cx);
    SwitchMotion {
        from_checked: state.from_checked,
        generation: state.generation,
        duration: state.duration,
        animate: state.animate_generation == Some(state.generation),
    }
}

fn track_width_for(
    has_content: bool,
    content_width: Pixels,
    size: SwitchSizeTokens,
    border_width: Pixels,
) -> Pixels {
    if has_content {
        size.content_thumb_size
            + size.content_slot_gap
            + content_width
            + size.content_track_padding * 2.
            + border_width * 2.
    } else {
        size.track_width
    }
}

fn stable_content_width(
    checked_content: Option<&SwitchContent>,
    unchecked_content: Option<&SwitchContent>,
    size: SwitchSizeTokens,
) -> Pixels {
    let checked_width = content_required_width(checked_content, size);
    let unchecked_width = content_required_width(unchecked_content, size);
    if checked_width > unchecked_width {
        checked_width
    } else {
        unchecked_width
    }
}

fn content_required_width(content: Option<&SwitchContent>, size: SwitchSizeTokens) -> Pixels {
    match content.map(|content| &content.kind) {
        Some(SwitchContentKind::Text(_)) => size.content_edge_padding + size.content_max_text_width,
        Some(SwitchContentKind::Icon(_)) => size.content_icon_size,
        Some(SwitchContentKind::IconText { .. }) => {
            size.content_edge_padding
                + size.content_icon_size
                + size.content_gap
                + size.content_max_text_width
        }
        None => px(0.),
    }
}

fn thumb_offset(
    checked: bool,
    has_content: bool,
    content_width: Pixels,
    size: SwitchSizeTokens,
    border_width: Pixels,
) -> Pixels {
    if has_content {
        return if checked {
            content_width + size.content_slot_gap
        } else {
            px(0.)
        };
    }

    compact_thumb_offset(
        checked,
        size.track_width,
        size.thumb_size,
        size.track_padding,
        border_width,
    )
}

fn compact_thumb_offset(
    checked: bool,
    track_width: Pixels,
    thumb_size: Pixels,
    track_padding: Pixels,
    border_width: Pixels,
) -> Pixels {
    if checked {
        track_width - thumb_size - track_padding * 2. - border_width * 2.
    } else {
        px(0.)
    }
}

fn interpolate_pixels(from: Pixels, to: Pixels, delta: f32) -> Pixels {
    from + (to - from) * delta
}

fn switch_ease_out_cubic(delta: f32) -> f32 {
    let delta = delta.clamp(0., 1.);
    1. - (1. - delta).powi(3)
}

fn content_opacities(from_checked: bool, checked: bool, delta: f32) -> (f32, f32) {
    if from_checked == checked {
        return if checked { (1., 0.) } else { (0., 1.) };
    }

    let delta = delta.clamp(0., 1.);
    let outgoing = 1. - (delta * 2.).min(1.);
    let incoming = ((delta - 0.5) * 2.).clamp(0., 1.);
    if checked {
        (incoming, outgoing)
    } else {
        (outgoing, incoming)
    }
}

#[derive(Clone, Copy)]
struct ContentRenderStyle {
    size: SwitchSizeTokens,
    content_width: Pixels,
    motion: SwitchMotion,
    checked: bool,
}

fn render_content_layer(
    checked_content: Option<SwitchContent>,
    unchecked_content: Option<SwitchContent>,
    style: ContentRenderStyle,
) -> AnyElement {
    let checked_region = render_content_region(checked_content, true, style);
    let unchecked_region = render_content_region(unchecked_content, false, style);

    div()
        .absolute()
        .inset_0()
        .child(checked_region)
        .child(unchecked_region)
        .into_any_element()
}

fn render_content_region(
    content: Option<SwitchContent>,
    for_checked: bool,
    style: ContentRenderStyle,
) -> AnyElement {
    let region = div()
        .absolute()
        .inset_0()
        .flex()
        .flex_none()
        .items_center()
        .p(style.size.content_track_padding)
        .when(for_checked, |this| this.justify_start())
        .when(!for_checked, |this| this.justify_end())
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .when(for_checked, |this| this.justify_start())
                .when(!for_checked, |this| this.justify_end())
                .w(style.content_width)
                .h_full()
                .overflow_hidden()
                .when_some(content, |this, content| {
                    this.child(render_switch_content(content, for_checked, style.size))
                }),
        );

    let final_opacity = if for_checked == style.checked { 1. } else { 0. };
    if style.motion.animate {
        let animation_name = if for_checked {
            "checked-content-motion"
        } else {
            "unchecked-content-motion"
        };
        region
            .with_animation(
                ElementId::named_usize(animation_name, style.motion.generation),
                Animation::new(style.motion.duration).with_easing(switch_ease_out_cubic),
                move |this, delta| {
                    let (checked_opacity, unchecked_opacity) =
                        content_opacities(style.motion.from_checked, style.checked, delta);
                    this.opacity(if for_checked {
                        checked_opacity
                    } else {
                        unchecked_opacity
                    })
                },
            )
            .into_any_element()
    } else {
        region.opacity(final_opacity).into_any_element()
    }
}

fn render_loading_indicator(
    indicator_id: ElementId,
    animation_id: ElementId,
    accessible_label: Option<SharedString>,
    size: Pixels,
    color: gpui::Hsla,
) -> AnyElement {
    div()
        .id(indicator_id)
        .role(Role::ProgressIndicator)
        .when_some(accessible_label, |this, label| this.aria_label(label))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(size)
        .child(
            svg()
                .path("components/button/loading.svg")
                .size(size)
                .text_color(color)
                .with_animation(
                    animation_id,
                    Animation::new(SWITCH_LOADING_SPINNER_DURATION).repeat(),
                    |icon, delta| {
                        icon.with_transformation(Transformation::rotate(percentage(delta)))
                    },
                ),
        )
        .into_any_element()
}

fn render_switch_content(
    content: SwitchContent,
    for_checked: bool,
    size: SwitchSizeTokens,
) -> AnyElement {
    match content.kind {
        SwitchContentKind::Text(text) => div()
            .flex()
            .items_center()
            .when(for_checked, |this| this.pl(size.content_edge_padding))
            .when(!for_checked, |this| this.pr(size.content_edge_padding))
            .child(
                div()
                    .min_w_0()
                    .max_w(size.content_max_text_width)
                    .truncate()
                    .child(text),
            )
            .into_any_element(),
        SwitchContentKind::Icon(icon) => Icon::new(icon)
            .size(size.content_icon_size)
            .into_any_element(),
        SwitchContentKind::IconText { icon, text } => div()
            .flex()
            .min_w_0()
            .items_center()
            .gap(size.content_gap)
            .when(for_checked, |this| this.pl(size.content_edge_padding))
            .when(!for_checked, |this| this.pr(size.content_edge_padding))
            .child(Icon::new(icon).size(size.content_icon_size))
            .child(
                div()
                    .min_w_0()
                    .max_w(size.content_max_text_width)
                    .truncate()
                    .child(text),
            )
            .into_any_element(),
    }
}

fn apply_interaction(
    element: gpui::Stateful<gpui::Div>,
    interaction: SwitchInteraction,
) -> gpui::Stateful<gpui::Div> {
    let SwitchInteraction {
        disabled,
        busy,
        on_click,
        cursor_style,
        hover,
        pressed,
        focused,
        focus_width,
    } = interaction;
    let has_handler = on_click.is_some();
    element
        .when(disabled, |this| {
            this.cursor(CursorStyle::OperationNotAllowed)
        })
        .when(!disabled, |this| {
            this.cursor(button::resolved_cursor_style(false, busy, cursor_style))
                .tab_index(0)
                .focus_visible(move |style| {
                    style
                        .border(focus_width)
                        .border_color(focused.track_border)
                        .text_color(focused.content)
                })
        })
        .when(!disabled && !busy, |this| {
            this.hover(move |style| style.text_color(hover.label))
                .active(move |style| style.text_color(pressed.label))
        })
        .when(busy, |this| {
            this.on_mouse_down(MouseButton::Left, |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(|_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .capture_key_down(|event, window, cx| {
                if is_plain_key(event, "enter") || is_plain_key(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
            .capture_key_up(|event, window, cx| {
                if is_plain_key_up(event, "enter") || is_plain_key_up(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
        })
        .when_some(on_click, |this, handler| {
            this.on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    (handler)(event, window, cx);
                })
        })
        .when(has_handler, |this| {
            this.capture_key_down(|event, window, cx| {
                if is_plain_key(event, "enter") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
            .capture_key_up(|event, window, cx| {
                if is_plain_key_up(event, "enter") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
        })
}

pub(crate) const fn toggled_state(checked: bool) -> Toggled {
    if checked {
        Toggled::True
    } else {
        Toggled::False
    }
}

fn is_plain_key(event: &KeyDownEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

fn is_plain_key_up(event: &KeyUpEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

#[cfg(test)]
#[path = "../tests/unit/switch.rs"]
mod tests;
