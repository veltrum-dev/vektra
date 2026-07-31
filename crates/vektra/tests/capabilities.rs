use gpui::{
    ClickEvent, Context, InteractiveElement, IntoElement, KeyUpEvent, Keystroke, Modifiers,
    ParentElement, Render, Styled, TestAppContext, Window, div, point, px,
};
use vektra::{Button, Clickable, Disableable, IconButton, IconSource};

#[test]
fn button_accepts_original_on_click_callback() {
    let _button = Button::new("button-original").on_click(|_, _, _| {});
}

#[test]
fn icon_button_accepts_original_on_click_callback() {
    let _button = IconButton::new("icon-original", IconSource::asset("icons/settings.svg"))
        .on_click(|_, _, _| {});
}

#[test]
fn button_and_icon_button_implement_disableable() {
    fn disable<C: Disableable>(component: C) -> C {
        component.disabled(true)
    }

    let _button = disable(Button::new("button-disableable"));
    let _icon_button = disable(IconButton::new(
        "icon-disableable",
        IconSource::asset("icons/settings.svg"),
    ));
}

#[test]
fn button_and_icon_button_implement_clickable() {
    fn clickable<C: Clickable>(component: C) -> C {
        component.on_click(|_, _, _| {})
    }

    let _button = clickable(Button::new("button-clickable"));
    let _icon_button = clickable(IconButton::new(
        "icon-clickable",
        IconSource::asset("icons/settings.svg"),
    ));
}

#[derive(Debug, Clone, Copy)]
enum Target {
    Button,
    IconButton,
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
}

impl CapabilityView {
    fn new(target: Target, disabled: bool) -> Self {
        Self {
            target,
            disabled,
            count: 0,
            notify_count: 0,
            saw_keyboard_event: false,
        }
    }

    fn record(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        self.notify_count += 1;
        self.saw_keyboard_event = event.is_keyboard();
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
            cx.simulate_keystrokes("enter");
        }
        Activation::Space => {
            cx.update(|window, cx| window.focus_next(cx));
            cx.simulate_event(KeyUpEvent {
                keystroke: Keystroke::parse("space").unwrap(),
            });
        }
    }
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
