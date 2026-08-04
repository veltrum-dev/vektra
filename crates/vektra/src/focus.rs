use gpui::{
    App, Context, Div, ElementId, Entity, FocusHandle, InteractiveElement, Stateful, Subscription,
    Window,
};
use std::rc::Rc;

pub(crate) type FocusHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub(crate) struct FocusState {
    focus_handle: FocusHandle,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
    observer_focus: Option<FocusHandler>,
    observer_blur: Option<FocusHandler>,
    _focus_subscription: Subscription,
    _blur_subscription: Subscription,
}

impl FocusState {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_index(0);
        let focus_subscription = cx.on_focus(&focus_handle, window, |this, window, cx| {
            let observer = this.observer_focus.clone();
            let handler = this.on_focus.clone();
            if let Some(observer) = observer {
                observer(window, cx);
            }
            if let Some(handler) = handler {
                handler(window, cx);
            }
        });
        let blur_subscription = cx.on_blur(&focus_handle, window, |this, window, cx| {
            let observer = this.observer_blur.clone();
            let handler = this.on_blur.clone();
            if let Some(observer) = observer {
                observer(window, cx);
            }
            if let Some(handler) = handler {
                handler(window, cx);
            }
        });

        Self {
            focus_handle,
            on_focus: None,
            on_blur: None,
            observer_focus: None,
            observer_blur: None,
            _focus_subscription: focus_subscription,
            _blur_subscription: blur_subscription,
        }
    }
}

pub(crate) fn state_for(
    id: &ElementId,
    tab_stop: bool,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FocusState> {
    let state = window.use_keyed_state((id.clone(), "focusable"), cx, FocusState::new);
    state.update(cx, |state, _| {
        state.focus_handle = state.focus_handle.clone().tab_stop(tab_stop).tab_index(0);
        state.on_focus = on_focus;
        state.on_blur = on_blur;
        state.observer_focus = None;
        state.observer_blur = None;
    });
    state
}

pub(crate) fn set_observers(
    state: &Entity<FocusState>,
    on_focus: FocusHandler,
    on_blur: FocusHandler,
    cx: &mut App,
) {
    state.update(cx, |state, _| {
        state.observer_focus = Some(on_focus);
        state.observer_blur = Some(on_blur);
    });
}

pub(crate) fn attach_interaction(
    element: Stateful<Div>,
    state: &Entity<FocusState>,
    focusable: bool,
    cx: &App,
) -> Stateful<Div> {
    if focusable {
        element.track_focus(&state.read(cx).focus_handle)
    } else {
        element
    }
}
