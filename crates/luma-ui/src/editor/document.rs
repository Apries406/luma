use std::{error::Error, fmt, ops::Range};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ByteOffset(usize);

impl ByteOffset {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for ByteOffset {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<ByteOffset> for usize {
    fn from(value: ByteOffset) -> Self {
        value.get()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Revision(u64);

impl Revision {
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEdit {
    range: Range<ByteOffset>,
    replacement: String,
}

impl TextEdit {
    pub(crate) fn new(range: Range<ByteOffset>, replacement: impl Into<String>) -> Self {
        Self {
            range,
            replacement: replacement.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditBatch {
    revision: Revision,
    edits: Vec<TextEdit>,
}

impl EditBatch {
    pub(crate) fn new(revision: Revision, edits: impl IntoIterator<Item = TextEdit>) -> Self {
        Self {
            revision,
            edits: edits.into_iter().collect(),
        }
    }

    pub(crate) fn single(revision: Revision, edit: TextEdit) -> Self {
        Self::new(revision, [edit])
    }

    pub(crate) fn single_range(
        revision: Revision,
        range: Range<ByteOffset>,
        replacement: impl Into<String>,
    ) -> Self {
        Self::single(revision, TextEdit::new(range, replacement))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ApplyError {
    EmptyBatch,
    StaleRevision {
        expected: Revision,
        actual: Revision,
    },
    InvalidRange {
        edit: usize,
        range: Range<ByteOffset>,
        document_len: ByteOffset,
    },
    NotCharBoundary {
        edit: usize,
        offset: ByteOffset,
    },
    Unsorted {
        edit: usize,
    },
    Overlapping {
        previous_edit: usize,
        edit: usize,
    },
    RevisionExhausted,
}

impl fmt::Display for ApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => formatter.write_str("edit batch is empty"),
            Self::StaleRevision { expected, actual } => write!(
                formatter,
                "edit batch revision {} does not match document revision {}",
                actual.get(),
                expected.get()
            ),
            Self::InvalidRange {
                edit,
                range,
                document_len,
            } => write!(
                formatter,
                "edit {edit} range {}..{} is invalid for document length {}",
                range.start.get(),
                range.end.get(),
                document_len.get()
            ),
            Self::NotCharBoundary { edit, offset } => write!(
                formatter,
                "edit {edit} offset {} is not a UTF-8 character boundary",
                offset.get()
            ),
            Self::Unsorted { edit } => {
                write!(formatter, "edit {edit} is not sorted by range")
            }
            Self::Overlapping {
                previous_edit,
                edit,
            } => write!(formatter, "edits {previous_edit} and {edit} overlap"),
            Self::RevisionExhausted => formatter.write_str("document revision is exhausted"),
        }
    }
}

impl Error for ApplyError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Document {
    text: String,
    line_starts: Vec<ByteOffset>,
    revision: Revision,
}

impl Document {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        let text = normalize_line_endings(text.into());
        let line_starts = line_starts(&text);
        Self {
            text,
            line_starts,
            revision: Revision::default(),
        }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn len(&self) -> ByteOffset {
        ByteOffset::new(self.text.len())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(crate) const fn revision(&self) -> Revision {
        self.revision
    }

    #[cfg(test)]
    pub(crate) fn line_starts(&self) -> &[ByteOffset] {
        &self.line_starts
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub(crate) fn line_start(&self, line: usize) -> Option<ByteOffset> {
        self.line_starts.get(line).copied()
    }

    pub(crate) fn line_index(&self, offset: ByteOffset) -> Option<usize> {
        if offset > self.len() || !self.is_char_boundary(offset) {
            return None;
        }
        Some(self.line_starts.partition_point(|start| *start <= offset) - 1)
    }

    pub(crate) fn is_char_boundary(&self, offset: ByteOffset) -> bool {
        self.text.is_char_boundary(offset.get())
    }

    pub(crate) fn replace_all(&self, text: impl Into<String>) -> EditBatch {
        EditBatch::single(
            self.revision,
            TextEdit::new(ByteOffset::new(0)..self.len(), text),
        )
    }

    pub(crate) fn apply(&mut self, batch: EditBatch) -> Result<Revision, ApplyError> {
        self.validate(&batch)?;
        let revision = Revision(
            self.revision
                .0
                .checked_add(1)
                .ok_or(ApplyError::RevisionExhausted)?,
        );
        let edits: Vec<_> = batch
            .edits
            .into_iter()
            .map(|edit| (edit.range, normalize_line_endings(edit.replacement)))
            .collect();
        let mut text = self.text.clone();
        for (range, replacement) in edits.into_iter().rev() {
            text.replace_range(range.start.get()..range.end.get(), &replacement);
        }

        self.line_starts = line_starts(&text);
        self.text = text;
        self.revision = revision;
        Ok(revision)
    }

    fn validate(&self, batch: &EditBatch) -> Result<(), ApplyError> {
        if batch.edits.is_empty() {
            return Err(ApplyError::EmptyBatch);
        }
        if batch.revision != self.revision {
            return Err(ApplyError::StaleRevision {
                expected: self.revision,
                actual: batch.revision,
            });
        }

        let mut previous: Option<&Range<ByteOffset>> = None;
        for (index, edit) in batch.edits.iter().enumerate() {
            let range = &edit.range;
            if range.start > range.end || range.end > self.len() {
                return Err(ApplyError::InvalidRange {
                    edit: index,
                    range: range.clone(),
                    document_len: self.len(),
                });
            }
            for offset in [range.start, range.end] {
                if !self.is_char_boundary(offset) {
                    return Err(ApplyError::NotCharBoundary {
                        edit: index,
                        offset,
                    });
                }
            }
            if let Some(previous) = previous {
                if (range.start, range.end) < (previous.start, previous.end) {
                    return Err(ApplyError::Unsorted { edit: index });
                }
                if previous.end > range.start {
                    return Err(ApplyError::Overlapping {
                        previous_edit: index - 1,
                        edit: index,
                    });
                }
            }
            previous = Some(range);
        }
        Ok(())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new(String::new())
    }
}

fn normalize_line_endings(text: String) -> String {
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text
    }
}

fn line_starts(text: &str) -> Vec<ByteOffset> {
    std::iter::once(ByteOffset::new(0))
        .chain(
            text.match_indices('\n')
                .map(|(index, _)| ByteOffset::new(index + 1)),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset(value: usize) -> ByteOffset {
        ByteOffset::new(value)
    }

    #[test]
    fn normalizes_line_endings_and_indexes_lines() {
        let document = Document::new("one\r\ntwo\rthree\n");

        assert_eq!(document.text(), "one\ntwo\nthree\n");
        assert_eq!(
            document.line_starts(),
            [offset(0), offset(4), offset(8), offset(14)]
        );
        assert_eq!(document.line_count(), 4);
        assert_eq!(document.line_index(offset(7)), Some(1));
        assert_eq!(document.line_index(offset(14)), Some(3));
        assert_eq!(document.line_index(offset(15)), None);
    }

    #[test]
    fn applies_multiple_edits_against_one_revision() {
        let mut document = Document::new("alpha beta gamma");
        let revision = document.revision();
        let batch = EditBatch::new(
            revision,
            [
                TextEdit::new(offset(0)..offset(5), "A\r\nline"),
                TextEdit::new(offset(6)..offset(10), "B"),
                TextEdit::new(offset(16)..offset(16), "!"),
            ],
        );

        let next_revision = document.apply(batch).unwrap();

        assert_eq!(document.text(), "A\nline B gamma!");
        assert_eq!(document.line_starts(), [offset(0), offset(2)]);
        assert!(next_revision > revision);
        assert_eq!(document.revision(), next_revision);
    }

    #[test]
    fn rejects_a_stale_revision_without_modifying_document() {
        let mut document = Document::new("abc");
        let stale_revision = document.revision();
        document
            .apply(EditBatch::single(
                stale_revision,
                TextEdit::new(offset(3)..offset(3), "d"),
            ))
            .unwrap();
        let before = document.clone();

        let result = document.apply(EditBatch::single(
            stale_revision,
            TextEdit::new(offset(0)..offset(1), "A"),
        ));

        assert!(matches!(result, Err(ApplyError::StaleRevision { .. })));
        assert_eq!(document, before);
    }

    #[test]
    fn rejects_invalid_utf8_boundary_without_modifying_document() {
        let mut document = Document::new("a😀b");
        let before = document.clone();
        let batch = EditBatch::single(
            document.revision(),
            TextEdit::new(offset(2)..offset(5), "x"),
        );

        assert!(matches!(
            document.apply(batch),
            Err(ApplyError::NotCharBoundary { .. })
        ));
        assert_eq!(document, before);
    }

    #[test]
    fn rejects_overlapping_or_unsorted_edits_without_modifying_document() {
        let mut document = Document::new("abcdef");
        let before = document.clone();
        let overlapping = EditBatch::new(
            document.revision(),
            [
                TextEdit::new(offset(1)..offset(4), "x"),
                TextEdit::new(offset(3)..offset(5), "y"),
            ],
        );

        assert!(matches!(
            document.apply(overlapping),
            Err(ApplyError::Overlapping { .. })
        ));
        assert_eq!(document, before);

        let unsorted = EditBatch::new(
            document.revision(),
            [
                TextEdit::new(offset(4)..offset(5), "x"),
                TextEdit::new(offset(1)..offset(2), "y"),
            ],
        );
        assert!(matches!(
            document.apply(unsorted),
            Err(ApplyError::Unsorted { .. })
        ));
        assert_eq!(document, before);
    }
}
