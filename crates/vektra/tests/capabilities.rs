use gpui::{
    ClickEvent, Context, InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke,
    Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div, point, px,
};
use vektra::{
    Button, Changeable, Checkbox, Clickable, ComponentSize, Disableable, IconButton, IconSource,
    Sizable, Switch,
};

#[test]
fn button_accepts_original_on_click_callback() {
    let _button = Button::new("button-original").on_click(|_, _, _| {});
}

#[test]
fn button_activity_and_selected_builders_are_publicly_chainable() {
    let _button = Button::new("button-state-builders")
        .loading(true)
        .progress(0.5)
        .loading(false)
        .selected(false)
        .selected(true);
}

#[test]
fn icon_button_accepts_original_on_click_callback() {
    let _button = IconButton::new("icon-original", IconSource::asset("icons/settings.svg"))
        .on_click(|_, _, _| {});
}

#[test]
fn interactive_components_implement_disableable() {
    fn disable<C: Disableable>(component: C) -> C {
        component.disabled(true)
    }

    let _button = disable(Button::new("button-disableable"));
    let _icon_button = disable(IconButton::new(
        "icon-disableable",
        IconSource::asset("icons/settings.svg"),
    ));
    let _checkbox = disable(Checkbox::new("checkbox-disableable"));
    let _switch = disable(Switch::new("switch-disableable"));
}

#[test]
fn components_with_size_api_implement_sizable() {
    fn sizable<C: Sizable>(component: C) -> C {
        component.size(ComponentSize::Sm)
    }

    let _button = sizable(Button::new("button-sizable"));
    let _icon_button = sizable(IconButton::new(
        "icon-sizable",
        IconSource::asset("icons/settings.svg"),
    ));
    let _checkbox = sizable(Checkbox::new("checkbox-sizable"));
    let _switch = sizable(Switch::new("switch-sizable"));
}

#[test]
fn components_with_standard_activation_entry_implement_clickable() {
    fn clickable<C: Clickable>(component: C) -> C {
        component.on_click(|_, _, _| {})
    }

    let _button = clickable(Button::new("button-clickable"));
    let _icon_button = clickable(IconButton::new(
        "icon-clickable",
        IconSource::asset("icons/settings.svg"),
    ));
    let _switch = clickable(Switch::new("switch-clickable"));
}

#[test]
fn controlled_components_implement_changeable_with_inherent_forwarding() {
    fn changeable<C: Changeable<bool>>(component: C) -> C {
        component.on_change(|_, _, _| {})
    }

    let _checkbox = changeable(Checkbox::new("checkbox-changeable").on_change(|_, _, _| {}));
    let _switch = changeable(Switch::new("switch-changeable").on_change(|_, _, _| {}));
}

struct GenericChangeableView {
    values: Vec<bool>,
}

fn bind_bool_change<C: Changeable<bool>>(component: C, cx: &Context<GenericChangeableView>) -> C {
    component.on_change_in(cx, |this, value, _, cx| {
        this.values.push(value);
        cx.notify();
    })
}

impl Render for GenericChangeableView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("generic-changeable-root")
            .size(px(180.))
            .child(bind_bool_change(
                Checkbox::new("generic-changeable").label("泛型变化"),
                cx,
            ))
    }
}

#[gpui::test]
fn generic_changeable_on_change_in_uses_the_same_behavior(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| GenericChangeableView { values: Vec::new() });
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    assert_eq!(view.read_with(cx, |view, _| view.values.clone()), [true]);
}

#[derive(Debug, Clone, Copy)]
enum Target {
    Button,
    IconButton,
    Checkbox,
}

#[derive(Debug, Clone, Copy)]
enum Activation {
    Mouse,
    Enter,
    Space,
}

struct CapabilityView {
    target: Target,
    disabled: bool,
    count: usize,
    notify_count: usize,
    saw_keyboard_event: bool,
    last_checkbox_next: Option<bool>,
}

impl CapabilityView {
    fn new(target: Target, disabled: bool) -> Self {
        Self {
            target,
            disabled,
            count: 0,
            notify_count: 0,
            saw_keyboard_event: false,
            last_checkbox_next: None,
        }
    }

    fn record(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        self.notify_count += 1;
        self.saw_keyboard_event = event.is_keyboard();
        window.prevent_default();
        cx.notify();
    }

    fn record_checkbox(&mut self, next_checked: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        self.notify_count += 1;
        self.last_checkbox_next = Some(next_checked);
        window.prevent_default();
        cx.notify();
    }
}

impl Render for CapabilityView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div().id("capability-root").size(px(180.));
        match self.target {
            Target::Button => root.child(
                Button::new("target")
                    .label("Hit")
                    .width(px(120.))
                    .disabled(self.disabled)
                    .on_click_in(cx, |this, event, window, cx| {
                        this.record(event, window, cx);
                    }),
            ),
            Target::IconButton => root.child(
                IconButton::new("target", IconSource::asset("icons/settings.svg"))
                    .aria_label("设置")
                    .disabled(self.disabled)
                    .on_click_in(cx, |this, event, window, cx| {
                        this.record(event, window, cx);
                    }),
            ),
            Target::Checkbox => root.child(
                Checkbox::new("target")
                    .label("接受条款")
                    .disabled(self.disabled)
                    .on_change_in(cx, |this, next_checked, window, cx| {
                        this.record_checkbox(next_checked, window, cx);
                    }),
            ),
        }
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn activate(cx: &mut gpui::VisualTestContext, activation: Activation) {
    match activation {
        Activation::Mouse => cx.simulate_click(point(px(18.), px(18.)), Modifiers::none()),
        Activation::Enter => {
            cx.update(|window, cx| window.focus_next(cx));
            simulate_key_cycle(cx, "enter");
        }
        Activation::Space => {
            cx.update(|window, cx| window.focus_next(cx));
            simulate_key_cycle(cx, "space");
        }
    }
}

fn simulate_key_cycle(cx: &mut gpui::VisualTestContext, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}

fn run_activation(
    cx: &mut TestAppContext,
    target: Target,
    disabled: bool,
    activation: Activation,
) -> (usize, usize, bool) {
    let (view, cx) = cx.add_window_view(|_, _| CapabilityView::new(target, disabled));
    draw(cx);
    activate(cx, activation);
    view.read_with(cx, |view, _| {
        (view.count, view.notify_count, view.saw_keyboard_event)
    })
}

fn run_checkbox_activation(
    cx: &mut TestAppContext,
    disabled: bool,
    activation: Activation,
) -> (usize, usize, Option<bool>) {
    let (view, cx) = cx.add_window_view(|_, _| CapabilityView::new(Target::Checkbox, disabled));
    draw(cx);
    activate(cx, activation);
    view.read_with(cx, |view, _| {
        (view.count, view.notify_count, view.last_checkbox_next)
    })
}

#[gpui::test]
fn button_on_click_in_mouse_updates_entity_and_notifies(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, false, Activation::Mouse),
        (1, 1, false)
    );
}

#[gpui::test]
fn icon_button_on_click_in_mouse_updates_entity_and_notifies(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, false, Activation::Mouse),
        (1, 1, false)
    );
}

#[gpui::test]
fn button_on_click_in_enter_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, false, Activation::Enter),
        (1, 1, true)
    );
}

#[gpui::test]
fn icon_button_on_click_in_enter_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, false, Activation::Enter),
        (1, 1, true)
    );
}

#[gpui::test]
fn button_on_click_in_space_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, false, Activation::Space),
        (1, 1, true)
    );
}

#[gpui::test]
fn icon_button_on_click_in_space_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, false, Activation::Space),
        (1, 1, true)
    );
}

#[gpui::test]
fn checkbox_on_change_in_mouse_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, false, Activation::Mouse),
        (1, 1, Some(true))
    );
}

#[gpui::test]
fn checkbox_on_change_in_space_updates_entity(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, false, Activation::Space),
        (1, 1, Some(true))
    );
}

#[gpui::test]
fn checkbox_enter_does_not_activate(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, false, Activation::Enter),
        (0, 0, None)
    );
}

#[gpui::test]
fn button_disabled_blocks_mouse(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, true, Activation::Mouse),
        (0, 0, false)
    );
}

#[gpui::test]
fn button_disabled_blocks_enter(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, true, Activation::Enter),
        (0, 0, false)
    );
}

#[gpui::test]
fn button_disabled_blocks_space(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::Button, true, Activation::Space),
        (0, 0, false)
    );
}

#[gpui::test]
fn icon_button_disabled_blocks_mouse(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, true, Activation::Mouse),
        (0, 0, false)
    );
}

#[gpui::test]
fn icon_button_disabled_blocks_enter(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, true, Activation::Enter),
        (0, 0, false)
    );
}

#[gpui::test]
fn icon_button_disabled_blocks_space(cx: &mut TestAppContext) {
    assert_eq!(
        run_activation(cx, Target::IconButton, true, Activation::Space),
        (0, 0, false)
    );
}

#[gpui::test]
fn checkbox_disabled_blocks_mouse(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, true, Activation::Mouse),
        (0, 0, None)
    );
}

#[gpui::test]
fn checkbox_disabled_blocks_space(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, true, Activation::Space),
        (0, 0, None)
    );
}

#[gpui::test]
fn checkbox_disabled_blocks_enter(cx: &mut TestAppContext) {
    assert_eq!(
        run_checkbox_activation(cx, true, Activation::Enter),
        (0, 0, None)
    );
}

#[gpui::test]
fn checkbox_modified_space_does_not_activate(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| CapabilityView::new(Target::Checkbox, false));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    simulate_key_cycle(cx, "cmd-space");

    assert_eq!(view.read_with(cx, |view, _| view.count), 0);
}

#[gpui::test]
fn checkbox_focus_change_between_space_down_and_up_cancels_activation(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| CapabilityView::new(Target::Checkbox, false));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    let keystroke = Keystroke::parse("space").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.update(|window, _| window.blur());
    cx.simulate_event(KeyUpEvent { keystroke });

    assert_eq!(view.read_with(cx, |view, _| view.count), 0);
}
