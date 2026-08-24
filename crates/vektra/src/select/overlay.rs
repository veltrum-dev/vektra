//! Select Popup 的视口约束、翻转与 deferred overlay 布局。

use super::*;

pub(super) struct SelectPopupOverlay {
    pub(super) body: Option<AnyElement>,
    pub(super) trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub(super) viewport_bounds: Bounds<Pixels>,
    pub(super) anchor_gap: Pixels,
    pub(super) viewport_padding: Pixels,
    pub(super) max_height: Pixels,
    pub(super) preferred_height: Pixels,
}

pub(super) struct PopupLayout {
    body: Option<AnyElement>,
    origin: Point<Pixels>,
}

impl IntoElement for SelectPopupOverlay {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectPopupOverlay {
    type RequestLayoutState = PopupLayout;
    type PrepaintState = AnyElement;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let trigger = self.trigger_bounds.get();
        let safe_top = self.viewport_bounds.top() + self.viewport_padding;
        let safe_bottom = self.viewport_bounds.bottom() - self.viewport_padding;
        let below = (safe_bottom - trigger.bottom() - self.anchor_gap).max(Pixels::ZERO);
        let above = (trigger.top() - safe_top - self.anchor_gap).max(Pixels::ZERO);
        let target_height = self.preferred_height.min(self.max_height);
        let open_above = below < target_height.min(above) && above > below;
        let available_height = target_height.min(if open_above { above } else { below });
        let available_width =
            (self.viewport_bounds.size.width - self.viewport_padding * 2.).max(Pixels::ZERO);
        let popup_width = trigger.size.width.min(available_width);
        let mut body = self.body.take().expect("Select Popup 每帧只允许布局一次");
        let body_size = body.layout_as_root(
            Size {
                width: AvailableSpace::Definite(popup_width),
                height: AvailableSpace::Definite(available_height),
            },
            window,
            cx,
        );
        let safe_left = self.viewport_bounds.left() + self.viewport_padding;
        let x = trigger
            .left()
            .min(self.viewport_bounds.right() - self.viewport_padding - body_size.width)
            .max(safe_left);
        let y = if open_above {
            trigger.top() - self.anchor_gap - body_size.height
        } else {
            trigger.bottom() + self.anchor_gap
        };
        let layout_id = window.request_layout(Style::default(), [], cx);
        (
            layout_id,
            PopupLayout {
                body: Some(body),
                origin: point(x, y.max(safe_top)),
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let mut body = layout.body.take().expect("Select Popup 必须先完成布局");
        body.prepaint_at(layout.origin, window, cx);
        body
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        body: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        body.paint(window, cx);
    }
}
