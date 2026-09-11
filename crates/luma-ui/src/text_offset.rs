use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn utf16_to_byte_offset(content: &str, offset: usize) -> usize {
    let mut byte_offset = 0;
    let mut utf16_offset = 0;
    for character in content.chars() {
        if utf16_offset >= offset {
            break;
        }
        utf16_offset += character.len_utf16();
        byte_offset += character.len_utf8();
    }
    byte_offset
}

fn utf16_to_byte_offset_left(content: &str, offset: usize) -> usize {
    let mut byte_offset = 0;
    let mut utf16_offset = 0;
    for character in content.chars() {
        let next_utf16_offset = utf16_offset + character.len_utf16();
        if offset < next_utf16_offset {
            break;
        }
        utf16_offset = next_utf16_offset;
        byte_offset += character.len_utf8();
    }
    byte_offset
}

pub(crate) fn byte_to_utf16_offset(content: &str, offset: usize) -> usize {
    let mut utf16_offset = 0;
    let mut byte_offset = 0;
    for character in content.chars() {
        if byte_offset >= offset {
            break;
        }
        byte_offset += character.len_utf8();
        utf16_offset += character.len_utf16();
    }
    utf16_offset
}

pub(crate) fn byte_range_to_utf16(content: &str, range: &Range<usize>) -> Range<usize> {
    byte_to_utf16_offset(content, range.start)..byte_to_utf16_offset(content, range.end)
}

pub(crate) fn utf16_range_to_bytes(content: &str, range: &Range<usize>) -> Range<usize> {
    if range.is_empty() {
        let offset = utf16_to_byte_offset(content, range.start);
        return offset..offset;
    }
    utf16_to_byte_offset_left(content, range.start)..utf16_to_byte_offset(content, range.end)
}

pub(crate) fn utf16_range_to_bytes_in(
    content: &str,
    base: &Range<usize>,
    range: &Range<usize>,
) -> Range<usize> {
    let relative = utf16_range_to_bytes(&content[base.clone()], range);
    base.start + relative.start..base.start + relative.end
}

pub(crate) fn previous_grapheme_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .rev()
        .find_map(|(index, _)| (index < offset).then_some(index))
        .unwrap_or(0)
}

pub(crate) fn next_grapheme_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .find_map(|(index, _)| (index > offset).then_some(index))
        .unwrap_or(content.len())
}

pub(crate) fn nearest_grapheme_boundary(content: &str, offset: usize) -> usize {
    let offset = offset.min(content.len());
    if offset == content.len()
        || content
            .grapheme_indices(true)
            .any(|(index, _)| index == offset)
    {
        return offset;
    }
    let previous = previous_grapheme_boundary(content, offset + 1);
    let next = next_grapheme_boundary(content, offset);
    if offset - previous <= next - offset {
        previous
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_utf8_and_utf16_offsets_without_splitting_scalars() {
        let content = "a😀e\u{301}中";
        for (utf8, utf16) in [(0, 0), (1, 1), (5, 3), (8, 5), (11, 6)] {
            assert_eq!(byte_to_utf16_offset(content, utf8), utf16);
            assert_eq!(utf16_to_byte_offset(content, utf16), utf8);
        }
        assert_eq!(utf16_to_byte_offset(content, 2), 5);
        assert_eq!(byte_to_utf16_offset(content, 2), 3);
        assert_eq!(utf16_range_to_bytes(content, &(1..5)), 1..8);
        assert_eq!(byte_range_to_utf16(content, &(1..8)), 1..5);
    }

    #[test]
    fn expands_utf16_ranges_that_split_surrogate_pairs() {
        let content = "a😀b";
        assert_eq!(utf16_range_to_bytes(content, &(2..3)), 1..5);
        assert_eq!(utf16_range_to_bytes(content, &(2..2)), 5..5);
    }

    #[test]
    fn navigates_extended_grapheme_boundaries() {
        let content = "a\r\n👩‍💻e\u{301}中";
        let boundaries: Vec<_> = content
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain([content.len()])
            .collect();
        assert_eq!(boundaries, [0, 1, 3, 14, 17, 20]);
        assert_eq!(previous_grapheme_boundary(content, 14), 3);
        assert_eq!(next_grapheme_boundary(content, 3), 14);
        assert_eq!(nearest_grapheme_boundary(content, 8), 3);
        assert_eq!(nearest_grapheme_boundary(content, 12), 14);
    }
}
