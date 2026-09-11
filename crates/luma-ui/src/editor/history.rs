use super::session::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditKind {
    Typing,
    Barrier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HistorySnapshot {
    pub(crate) text: String,
    pub(crate) selection: Selection,
}

impl HistorySnapshot {
    pub(crate) fn new(text: impl Into<String>, selection: Selection) -> Self {
        Self {
            text: text.into(),
            selection,
        }
    }

    pub(crate) fn into_parts(self) -> (String, Selection) {
        (self.text, self.selection)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditTransaction {
    before: HistorySnapshot,
    after: HistorySnapshot,
    kind: EditKind,
}

impl EditTransaction {
    pub(crate) const fn new(
        before: HistorySnapshot,
        after: HistorySnapshot,
        kind: EditKind,
    ) -> Self {
        Self {
            before,
            after,
            kind,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditHistory {
    undo: Vec<EditTransaction>,
    redo: Vec<EditTransaction>,
    merge_typing: bool,
}

impl EditHistory {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(crate) fn record(&mut self, transaction: EditTransaction) -> bool {
        if transaction.before.text == transaction.after.text {
            return false;
        }

        self.redo.clear();
        if transaction.kind == EditKind::Typing
            && self.merge_typing
            && let Some(previous) = self.undo.last_mut()
            && previous.kind == EditKind::Typing
            && previous.after == transaction.before
        {
            previous.after = transaction.after;
            if previous.before.text == previous.after.text {
                self.undo.pop();
                self.merge_typing = false;
            }
            return true;
        }

        self.merge_typing = transaction.kind == EditKind::Typing;
        self.undo.push(transaction);
        true
    }

    pub(crate) fn barrier(&mut self) {
        self.merge_typing = false;
    }

    pub(crate) fn undo(&mut self) -> Option<HistorySnapshot> {
        self.merge_typing = false;
        let transaction = self.undo.pop()?;
        let target = transaction.before.clone();
        self.redo.push(transaction);
        Some(target)
    }

    pub(crate) fn redo(&mut self) -> Option<HistorySnapshot> {
        self.merge_typing = false;
        let transaction = self.redo.pop()?;
        let target = transaction.after.clone();
        self.undo.push(transaction);
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::document::ByteOffset;

    fn selection(anchor: usize, active: usize) -> Selection {
        Selection::new(ByteOffset::new(anchor), ByteOffset::new(active))
    }

    fn transaction(
        before_text: &str,
        before_selection: Selection,
        after_text: &str,
        after_selection: Selection,
        kind: EditKind,
    ) -> EditTransaction {
        EditTransaction::new(
            HistorySnapshot::new(before_text, before_selection),
            HistorySnapshot::new(after_text, after_selection),
            kind,
        )
    }

    #[test]
    fn undo_and_redo_restore_text_and_selection() {
        let before_selection = selection(1, 3);
        let after_selection = selection(2, 2);
        let mut history = EditHistory::new();
        history.record(transaction(
            "abc",
            before_selection,
            "axc",
            after_selection,
            EditKind::Barrier,
        ));

        let undo = history.undo().unwrap();
        assert_eq!(undo.text, "abc");
        assert_eq!(undo.selection, before_selection);

        let redo = history.redo().unwrap();
        assert_eq!(redo.text, "axc");
        assert_eq!(redo.selection, after_selection);
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut history = EditHistory::new();
        history.record(transaction(
            "",
            selection(0, 0),
            "a",
            selection(1, 1),
            EditKind::Typing,
        ));
        assert!(history.undo().is_some());
        assert!(history.can_redo());

        history.record(transaction(
            "",
            selection(0, 0),
            "b",
            selection(1, 1),
            EditKind::Typing,
        ));

        assert!(!history.can_redo());
        assert_eq!(history.undo().unwrap().text, "");
    }

    #[test]
    fn canceled_typing_group_does_not_cross_a_barrier() {
        let mut history = EditHistory::new();
        history.record(transaction(
            "",
            selection(0, 0),
            "a",
            selection(1, 1),
            EditKind::Typing,
        ));
        history.barrier();
        history.record(transaction(
            "a",
            selection(1, 1),
            "ab",
            selection(2, 2),
            EditKind::Typing,
        ));
        history.record(transaction(
            "ab",
            selection(2, 2),
            "a",
            selection(1, 1),
            EditKind::Typing,
        ));
        history.record(transaction(
            "a",
            selection(1, 1),
            "ac",
            selection(2, 2),
            EditKind::Typing,
        ));

        assert_eq!(history.undo.len(), 2);
        assert_eq!(history.undo().unwrap().text, "a");
        assert_eq!(history.undo().unwrap().text, "");
    }

    #[test]
    fn consecutive_typing_merges_until_a_barrier() {
        let mut history = EditHistory::new();
        history.record(transaction(
            "",
            selection(0, 0),
            "a",
            selection(1, 1),
            EditKind::Typing,
        ));
        history.record(transaction(
            "a",
            selection(1, 1),
            "ab",
            selection(2, 2),
            EditKind::Typing,
        ));

        assert_eq!(history.undo.len(), 1);
        assert_eq!(history.undo().unwrap().text, "");
        assert_eq!(history.redo().unwrap().text, "ab");

        history.barrier();
        history.record(transaction(
            "ab",
            selection(2, 2),
            "abc",
            selection(3, 3),
            EditKind::Typing,
        ));
        assert_eq!(history.undo.len(), 2);
        assert_eq!(history.undo().unwrap().text, "ab");
    }

    #[test]
    fn barrier_edits_never_merge() {
        let mut history = EditHistory::new();
        history.record(transaction(
            "a",
            selection(1, 1),
            "ab",
            selection(2, 2),
            EditKind::Barrier,
        ));
        history.record(transaction(
            "ab",
            selection(2, 2),
            "abc",
            selection(3, 3),
            EditKind::Barrier,
        ));

        assert_eq!(history.undo.len(), 2);
    }
}
