use gpui::{Bounds, Pixels, Point, Size, point};

#[derive(Clone, Copy, Debug)]
pub(super) struct LayoutViewport {
    document_revision: u64,
    scroll_offset: Point<Pixels>,
    viewport_size: Option<Size<Pixels>>,
    ensure_caret_visible: bool,
}

impl LayoutViewport {
    pub(super) fn new(document_revision: u64) -> Self {
        Self {
            document_revision,
            scroll_offset: Point::default(),
            viewport_size: None,
            ensure_caret_visible: true,
        }
    }

    pub(super) fn scroll_offset(&self) -> Point<Pixels> {
        self.scroll_offset
    }

    pub(super) fn needs_caret_reveal(
        &self,
        document_revision: u64,
        viewport_size: Size<Pixels>,
    ) -> bool {
        self.ensure_caret_visible
            || self.document_revision != document_revision
            || self.viewport_size != Some(viewport_size)
    }

    pub(super) fn request_caret_visible(&mut self) {
        self.ensure_caret_visible = true;
    }

    pub(super) fn finish_layout(
        &mut self,
        document_revision: u64,
        viewport_size: Size<Pixels>,
        scroll_offset: Point<Pixels>,
        caret_revealed: bool,
    ) -> bool {
        let changed = self.scroll_offset != scroll_offset;
        self.document_revision = document_revision;
        self.viewport_size = Some(viewport_size);
        self.scroll_offset = scroll_offset;
        if caret_revealed {
            self.ensure_caret_visible = false;
        }
        changed
    }
}

pub(super) fn reveal_scroll(
    offset: Point<Pixels>,
    target: Bounds<Pixels>,
    viewport: Bounds<Pixels>,
    max_scroll: Point<Pixels>,
) -> Point<Pixels> {
    let reveal_axis = |offset: Pixels,
                       target_start: Pixels,
                       target_end: Pixels,
                       viewport_start: Pixels,
                       viewport_end: Pixels,
                       max_scroll: Pixels| {
        let offset = if target_start < viewport_start {
            offset + viewport_start - target_start
        } else if target_end > viewport_end {
            offset - (target_end - viewport_end)
        } else {
            offset
        };
        offset.clamp(-max_scroll, Pixels::ZERO)
    };

    point(
        reveal_axis(
            offset.x,
            target.left(),
            target.right(),
            viewport.left(),
            viewport.right(),
            max_scroll.x,
        ),
        reveal_axis(
            offset.y,
            target.top(),
            target.bottom(),
            viewport.top(),
            viewport.bottom(),
            max_scroll.y,
        ),
    )
}

#[cfg(test)]
mod tests {
    use gpui::{px, size};

    use super::*;

    #[test]
    fn reveals_caret_minimally_on_both_axes() {
        let viewport = Bounds::new(point(px(10.), px(20.)), size(px(100.), px(40.)));
        let max_scroll = point(px(50.), px(30.));

        assert_eq!(
            reveal_scroll(
                point(px(-20.), px(-10.)),
                Bounds::new(point(px(125.), px(68.)), size(px(2.), px(22.))),
                viewport,
                max_scroll,
            ),
            point(px(-37.), px(-30.))
        );
        assert_eq!(
            reveal_scroll(
                point(px(-20.), px(-10.)),
                Bounds::new(point(px(40.), px(30.)), size(px(2.), px(12.))),
                viewport,
                max_scroll,
            ),
            point(px(-20.), px(-10.))
        );
    }

    #[test]
    fn revision_change_requests_caret_reveal() {
        let mut viewport = LayoutViewport::new(1);
        let viewport_size = size(px(100.), px(80.));
        assert!(viewport.needs_caret_reveal(1, viewport_size));
        assert!(!viewport.finish_layout(1, viewport_size, Point::default(), false));
        assert!(viewport.needs_caret_reveal(1, viewport_size));
        assert!(!viewport.finish_layout(1, viewport_size, Point::default(), true));
        assert!(!viewport.needs_caret_reveal(1, viewport_size));
        assert!(viewport.needs_caret_reveal(2, viewport_size));
        assert!(viewport.needs_caret_reveal(1, size(px(80.), px(60.))));
    }
}
