//! Input 单行文本的自定义布局、绘制与平台输入桥接。

use super::*;

pub(super) struct InputTextElement {
    pub(super) state: Entity<InputState>,
    pub(super) placeholder: SharedString,
    pub(super) colors: InputStateTokens,
    pub(super) caret_color: Hsla,
    pub(super) caret_width: Pixels,
    pub(super) caret_opacity: f32,
}

pub(super) struct InputTextPrepaint {
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
        let content = display.text.clone();
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
