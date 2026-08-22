//! Select 的 AccessKit 委托、稳定节点关联与 option 测量桥接。

use super::*;

/// 为锁定 GPUI 高层元素尚未暴露的 AccessKit disabled 属性提供私有委托层。
pub(super) struct DisabledA11y {
    inner: Stateful<Div>,
    disabled: bool,
    measured_bounds: Option<Rc<Cell<Bounds<Pixels>>>>,
    controlled_node_id: Option<gpui::accesskit::NodeId>,
}

impl DisabledA11y {
    pub(super) fn new(
        inner: Stateful<Div>,
        disabled: bool,
        measured_bounds: Option<Rc<Cell<Bounds<Pixels>>>>,
        _scroll_request: Option<()>,
    ) -> Self {
        Self {
            inner,
            disabled,
            measured_bounds,
            controlled_node_id: None,
        }
    }

    pub(super) fn controls(mut self, node_id: gpui::accesskit::NodeId) -> Self {
        self.controlled_node_id = Some(node_id);
        self
    }
}

impl IntoElement for DisabledA11y {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for DisabledA11y {
    type RequestLayoutState = <Stateful<Div> as Element>::RequestLayoutState;
    type PrepaintState = <Stateful<Div> as Element>::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Element::id(&self.inner)
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        self.inner.a11y_role()
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.inner.write_a11y_info(node);
        if self.disabled {
            node.set_disabled();
        } else {
            node.clear_disabled();
        }
        if let Some(controlled_node_id) = self.controlled_node_id {
            node.set_controls([controlled_node_id]);
        } else {
            node.clear_controls();
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        Element::a11y_synthetic_children(&mut self.inner, prepaint, builder);
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner
            .request_layout(global_id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if let Some(measured_bounds) = self.measured_bounds.as_ref()
            && measured_bounds.get() != bounds
        {
            measured_bounds.set(bounds);
            window.refresh();
        }
        self.inner
            .prepaint(global_id, inspector_id, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner.paint(
            global_id,
            inspector_id,
            bounds,
            layout,
            prepaint,
            window,
            cx,
        );
    }
}

pub(super) fn accesskit_node_id(global_id: &GlobalElementId) -> gpui::accesskit::NodeId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    global_id.hash(&mut hasher);
    gpui::accesskit::NodeId(hasher.finish())
}

pub(super) fn select_popup_node_id(id: ElementId, window: &mut Window) -> gpui::accesskit::NodeId {
    window.with_global_id(id, |_, window| {
        window.with_global_id(ElementId::from("vektra-select-popup"), |global_id, _| {
            accesskit_node_id(global_id)
        })
    })
}
