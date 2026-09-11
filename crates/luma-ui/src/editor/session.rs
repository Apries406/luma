use std::{error::Error, fmt, ops::Range};

use super::document::{ByteOffset, Document};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectionDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Selection {
    anchor: ByteOffset,
    active: ByteOffset,
}

impl Selection {
    pub(crate) const fn new(anchor: ByteOffset, active: ByteOffset) -> Self {
        Self { anchor, active }
    }

    pub(crate) const fn caret(offset: ByteOffset) -> Self {
        Self::new(offset, offset)
    }

    pub(crate) const fn anchor(self) -> ByteOffset {
        self.anchor
    }

    pub(crate) const fn active(self) -> ByteOffset {
        self.active
    }

    pub(crate) const fn direction(self) -> SelectionDirection {
        if self.active.get() < self.anchor.get() {
            SelectionDirection::Backward
        } else {
            SelectionDirection::Forward
        }
    }

    pub(crate) const fn is_collapsed(self) -> bool {
        self.anchor.get() == self.active.get()
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.is_collapsed()
    }

    pub(crate) const fn is_reversed(self) -> bool {
        matches!(self.direction(), SelectionDirection::Backward)
    }

    pub(crate) fn range(self) -> Range<ByteOffset> {
        self.anchor.min(self.active)..self.anchor.max(self.active)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TransactionToken(u64);

impl TransactionToken {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompositionState {
    marked_range: Range<ByteOffset>,
    transaction: TransactionToken,
}

impl CompositionState {
    pub(crate) fn new(marked_range: Range<ByteOffset>, transaction: TransactionToken) -> Self {
        Self {
            marked_range,
            transaction,
        }
    }

    pub(crate) fn marked_range(&self) -> Range<ByteOffset> {
        self.marked_range.clone()
    }

    pub(crate) const fn transaction(&self) -> TransactionToken {
        self.transaction
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionError {
    SelectionOutOfBounds,
    SelectionNotCharBoundary,
    CompositionRangeOutOfBounds,
    CompositionRangeNotCharBoundary,
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SelectionOutOfBounds => "selection is outside the document",
            Self::SelectionNotCharBoundary => "selection is not on UTF-8 character boundaries",
            Self::CompositionRangeOutOfBounds => "composition range is outside the document",
            Self::CompositionRangeNotCharBoundary => {
                "composition range is not on UTF-8 character boundaries"
            }
        })
    }
}

impl Error for SessionError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EditorSession {
    selection: Selection,
    preferred_horizontal_x: Option<f32>,
    composition: Option<CompositionState>,
}

impl EditorSession {
    pub(crate) fn new(document: &Document) -> Self {
        Self {
            selection: Selection::caret(document.len()),
            preferred_horizontal_x: None,
            composition: None,
        }
    }

    pub(crate) const fn selection(&self) -> Selection {
        self.selection
    }

    pub(crate) fn set_selection(
        &mut self,
        document: &Document,
        selection: Selection,
    ) -> Result<(), SessionError> {
        validate_selection(document, selection)?;
        self.selection = selection;
        Ok(())
    }

    pub(crate) const fn preferred_horizontal_x(&self) -> Option<f32> {
        self.preferred_horizontal_x
    }

    pub(crate) fn set_preferred_horizontal_x(&mut self, x: f32) {
        self.preferred_horizontal_x = Some(x);
    }

    pub(crate) fn clear_preferred_horizontal_x(&mut self) {
        self.preferred_horizontal_x = None;
    }

    pub(crate) const fn composition(&self) -> Option<&CompositionState> {
        self.composition.as_ref()
    }

    pub(crate) fn set_composition(
        &mut self,
        document: &Document,
        composition: CompositionState,
    ) -> Result<(), SessionError> {
        validate_range(document, &composition.marked_range).map_err(|error| match error {
            RangeError::OutOfBounds => SessionError::CompositionRangeOutOfBounds,
            RangeError::NotCharBoundary => SessionError::CompositionRangeNotCharBoundary,
        })?;
        self.composition = Some(composition);
        Ok(())
    }

    pub(crate) fn clear_composition(&mut self) -> Option<CompositionState> {
        self.composition.take()
    }
}

#[derive(Clone, Copy)]
enum RangeError {
    OutOfBounds,
    NotCharBoundary,
}

fn validate_selection(document: &Document, selection: Selection) -> Result<(), SessionError> {
    if selection.anchor > document.len() || selection.active > document.len() {
        return Err(SessionError::SelectionOutOfBounds);
    }
    if !document.is_char_boundary(selection.anchor) || !document.is_char_boundary(selection.active)
    {
        return Err(SessionError::SelectionNotCharBoundary);
    }
    Ok(())
}

fn validate_range(document: &Document, range: &Range<ByteOffset>) -> Result<(), RangeError> {
    if range.start > range.end || range.end > document.len() {
        return Err(RangeError::OutOfBounds);
    }
    if !document.is_char_boundary(range.start) || !document.is_char_boundary(range.end) {
        return Err(RangeError::NotCharBoundary);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset(value: usize) -> ByteOffset {
        ByteOffset::new(value)
    }

    #[test]
    fn selection_preserves_anchor_active_and_direction() {
        let forward = Selection::new(offset(1), offset(4));
        assert_eq!(forward.direction(), SelectionDirection::Forward);
        assert!(!forward.is_empty());
        assert!(!forward.is_reversed());
        assert_eq!(forward.range(), offset(1)..offset(4));

        let backward = Selection::new(offset(4), offset(1));
        assert_eq!(backward.direction(), SelectionDirection::Backward);
        assert!(backward.is_reversed());
        assert_eq!(backward.range(), offset(1)..offset(4));
    }

    #[test]
    fn selection_uses_normalized_utf8_offsets_across_emoji_and_crlf() {
        let document = Document::new("😀\r\n中");
        let mut session = EditorSession::new(&document);

        session
            .set_selection(&document, Selection::new(offset(8), offset(5)))
            .unwrap();

        assert_eq!(document.text(), "😀\n中");
        assert_eq!(
            session.selection().direction(),
            SelectionDirection::Backward
        );
        assert_eq!(session.selection().range(), offset(5)..offset(8));
    }

    #[test]
    fn session_rejects_non_boundary_selection_and_composition() {
        let document = Document::new("a😀b");
        let mut session = EditorSession::new(&document);
        let original = session.clone();

        assert_eq!(
            session.set_selection(&document, Selection::caret(offset(2))),
            Err(SessionError::SelectionNotCharBoundary)
        );
        assert_eq!(session, original);

        assert_eq!(
            session.set_composition(
                &document,
                CompositionState::new(offset(1)..offset(3), TransactionToken::new(7)),
            ),
            Err(SessionError::CompositionRangeNotCharBoundary)
        );
        assert_eq!(session, original);
    }
}
