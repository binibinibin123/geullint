use crate::{SourceKind, TextRange};

pub(crate) fn source_ranges(text: &str, source_kind: SourceKind) -> Vec<TextRange> {
    let ranges = match source_kind {
        SourceKind::PlainText => vec![TextRange {
            start: 0,
            end: text.len(),
        }],
        SourceKind::Markdown => markdown_ranges(text),
        SourceKind::Python => python_comment_ranges(text),
        SourceKind::JavaScript | SourceKind::TypeScript => javascript_comment_ranges(text),
        SourceKind::Rust => rust_comment_ranges(text),
    };
    exclude_identifier_ranges(
        text,
        &ranges,
        matches!(source_kind, SourceKind::PlainText | SourceKind::Markdown),
    )
}

fn exclude_identifier_ranges(
    text: &str,
    ranges: &[TextRange],
    protect_social_identifiers: bool,
) -> Vec<TextRange> {
    let mut filtered = Vec::new();

    for range in ranges {
        let source = &text[range.start..range.end];
        let mut prose_start = range.start;
        let mut token_start = None;

        for (relative_offset, character) in source
            .char_indices()
            .chain(std::iter::once((source.len(), ' ')))
        {
            if !character.is_whitespace() {
                token_start.get_or_insert(relative_offset);
                continue;
            }

            let Some(relative_start) = token_start.take() else {
                continue;
            };
            let token = &source[relative_start..relative_offset];
            let leading = token.len() - token.trim_start_matches(identifier_leading_wrapper).len();
            let trimmed_start = relative_start + leading;
            let trimmed = token[leading..].trim_end_matches(identifier_trailing_wrapper);
            if trimmed.is_empty() || !is_protected_identifier(trimmed, protect_social_identifiers) {
                continue;
            }

            let identifier_start = range.start + trimmed_start;
            let identifier_end = identifier_start + trimmed.len();
            push_non_empty_range(&mut filtered, prose_start, identifier_start);
            prose_start = identifier_end;
        }

        push_non_empty_range(&mut filtered, prose_start, range.end);
    }

    filtered
}

fn identifier_leading_wrapper(character: char) -> bool {
    matches!(
        character,
        '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '“' | '”' | '‘' | '’'
    )
}

fn identifier_trailing_wrapper(character: char) -> bool {
    identifier_leading_wrapper(character) || matches!(character, '.' | ',' | '!' | '?' | ';' | ':')
}

fn is_protected_identifier(token: &str, protect_social_identifiers: bool) -> bool {
    token.contains("://")
        || token.starts_with("www.")
        || token.contains('@')
        || token.contains('/')
        || token.contains('\\')
        || has_ascii_dot_suffix(token)
        || (protect_social_identifiers && token.len() > 1 && token.starts_with('#'))
}

fn has_ascii_dot_suffix(token: &str) -> bool {
    token.rsplit_once('.').is_some_and(|(prefix, suffix)| {
        if prefix.is_empty() {
            return false;
        }

        let ascii_end = suffix
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            .count();
        let identifier = &suffix[..ascii_end];
        let attached_korean = &suffix[ascii_end..];

        !identifier.is_empty()
            && identifier.len() <= 63
            && is_identifier_korean_suffix(attached_korean)
    })
}

fn is_identifier_korean_suffix(suffix: &str) -> bool {
    suffix
        .chars()
        .all(|character| ('가'..='힣').contains(&character))
}

fn markdown_ranges(text: &str) -> Vec<TextRange> {
    let mut ranges = Vec::new();
    let mut line_start = 0;
    let mut fence = None;
    let mut inline_delimiter = None;

    for line in text.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content = &text[line_start..line_end];

        if inline_delimiter.is_some() {
            push_non_code_markdown_ranges(content, line_start, &mut inline_delimiter, &mut ranges);
        } else if let Some(active_fence) = fence {
            if is_markdown_fence_close(content, active_fence) {
                fence = None;
            }
        } else if let Some(opening_fence) = markdown_fence_open(content) {
            fence = Some(opening_fence);
        } else if !is_indented_markdown_code(content) {
            push_non_code_markdown_ranges(content, line_start, &mut inline_delimiter, &mut ranges);
        }

        line_start = line_end;
    }

    ranges
}

#[derive(Clone, Copy)]
struct MarkdownFence {
    marker: u8,
    width: usize,
}

fn markdown_fence_open(line: &str) -> Option<MarkdownFence> {
    let body = markdown_line_body(line).as_bytes();
    let (indent, marker_start) = markdown_indent(body);
    if indent > 3 {
        return None;
    }

    let marker = *body.get(marker_start)?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let width = repeated_byte_width(body, marker_start, marker);
    if width < 3 {
        return None;
    }

    let remainder = &body[marker_start + width..];
    if marker == b'`' && remainder.contains(&b'`') {
        return None;
    }

    Some(MarkdownFence { marker, width })
}

fn is_markdown_fence_close(line: &str, fence: MarkdownFence) -> bool {
    let body = markdown_line_body(line).as_bytes();
    let (indent, marker_start) = markdown_indent(body);
    if indent > 3 || body.get(marker_start) != Some(&fence.marker) {
        return false;
    }

    let width = repeated_byte_width(body, marker_start, fence.marker);
    width >= fence.width
        && body[marker_start + width..]
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'))
}

fn is_indented_markdown_code(line: &str) -> bool {
    markdown_indent(markdown_line_body(line).as_bytes()).0 >= 4
}

fn markdown_line_body(line: &str) -> &str {
    let without_line_feed = line.strip_suffix('\n').unwrap_or(line);
    without_line_feed
        .strip_suffix('\r')
        .unwrap_or(without_line_feed)
}

fn markdown_indent(bytes: &[u8]) -> (usize, usize) {
    let mut columns = 0;
    let mut index = 0;
    while let Some(byte) = bytes.get(index) {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns += 4 - (columns % 4),
            _ => break,
        }
        index += 1;
    }
    (columns, index)
}

fn repeated_byte_width(bytes: &[u8], start: usize, byte: u8) -> usize {
    bytes[start..]
        .iter()
        .take_while(|candidate| **candidate == byte)
        .count()
}

fn push_non_code_markdown_ranges(
    line: &str,
    line_start: usize,
    inline_delimiter: &mut Option<usize>,
    ranges: &mut Vec<TextRange>,
) {
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut segment_start = inline_delimiter.is_none().then_some(line_start);

    while index < bytes.len() {
        if inline_delimiter.is_none() && bytes[index] == b'!' && bytes.get(index + 1) == Some(&b'[')
        {
            push_non_empty_range(
                ranges,
                segment_start.expect("prose has a start before an image marker"),
                line_start + index,
            );
            index += 1;
            segment_start = Some(line_start + index);
            continue;
        }

        if inline_delimiter.is_none()
            && bytes[index] == b'<'
            && markdown_autolink_starts(bytes, index)
        {
            push_non_empty_range(
                ranges,
                segment_start.expect("prose has a start before an autolink"),
                line_start + index,
            );

            let Some(destination_end) = bytes[index + 1..]
                .iter()
                .position(|byte| *byte == b'>')
                .map(|relative_end| index + relative_end + 2)
            else {
                return;
            };
            index = destination_end;
            segment_start = Some(line_start + destination_end);
            continue;
        }

        if inline_delimiter.is_none() && bytes[index] == b']' && bytes.get(index + 1) == Some(&b'(')
        {
            push_non_empty_range(
                ranges,
                segment_start.expect("prose has a start before a link destination"),
                line_start + index + 1,
            );

            let Some(destination_end) = markdown_link_destination_end(bytes, index + 1) else {
                return;
            };
            index = destination_end;
            segment_start = Some(line_start + destination_end);
            continue;
        }

        if bytes[index] != b'`'
            || (inline_delimiter.is_none() && markdown_backtick_is_escaped(bytes, index))
        {
            index += 1;
            continue;
        }

        let width = repeated_byte_width(bytes, index, b'`');
        let delimiter_end = index + width;
        match *inline_delimiter {
            Some(opening_width) if opening_width == width => {
                *inline_delimiter = None;
                segment_start = Some(line_start + delimiter_end);
            }
            Some(_) => {}
            None => {
                push_non_empty_range(
                    ranges,
                    segment_start.expect("prose has a start before inline code"),
                    line_start + index,
                );
                *inline_delimiter = Some(width);
                segment_start = None;
            }
        }
        index = delimiter_end;
    }

    if inline_delimiter.is_none() {
        push_non_empty_range(
            ranges,
            segment_start.expect("prose resumes after inline code"),
            line_start + line.len(),
        );
    }
}

fn markdown_autolink_starts(bytes: &[u8], opening_angle: usize) -> bool {
    let Some(first) = bytes.get(opening_angle + 1).copied() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    bytes[opening_angle + 2..]
        .iter()
        .take(32)
        .position(|byte| *byte == b':')
        .is_some_and(|relative_colon| {
            bytes[opening_angle + 2..opening_angle + 2 + relative_colon]
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-'))
        })
}

fn markdown_link_destination_end(bytes: &[u8], opening_parenthesis: usize) -> Option<usize> {
    let mut index = opening_parenthesis + 1;
    let mut depth = 1_usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth -= 1;
                index += 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            b'\r' | b'\n' => return None,
            _ => index += 1,
        }
    }

    None
}

fn markdown_backtick_is_escaped(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        cursor -= 1;
    }
    (index - cursor) % 2 == 1
}

fn python_comment_ranges(text: &str) -> Vec<TextRange> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if let Some((quote_index, delimiter)) = python_string_start(bytes, index) {
            let Some(end) = skip_python_string(bytes, quote_index, delimiter) else {
                break;
            };
            index = end;
            continue;
        }

        if bytes[index] == b'#' {
            let end = line_end(bytes, index);
            push_non_empty_range(&mut ranges, index, end);
            index = end;
        } else {
            index += 1;
        }
    }

    ranges
}

fn python_string_start(bytes: &[u8], index: usize) -> Option<(usize, u8)> {
    if let quote @ (b'\'' | b'"') = bytes[index] {
        return Some((index, quote));
    }

    if index > 0 && is_python_identifier_byte(bytes[index - 1]) {
        return None;
    }

    let mut cursor = index;
    while cursor < bytes.len()
        && cursor - index < 3
        && matches!(
            bytes[cursor],
            b'b' | b'B' | b'f' | b'F' | b'r' | b'R' | b'u' | b'U'
        )
    {
        cursor += 1;
    }
    if cursor == index || cursor - index > 2 {
        return None;
    }
    bytes
        .get(cursor)
        .copied()
        .filter(|byte| matches!(byte, b'\'' | b'"'))
        .map(|delimiter| (cursor, delimiter))
}

fn skip_python_string(bytes: &[u8], quote_index: usize, delimiter: u8) -> Option<usize> {
    let triple = bytes.get(quote_index..quote_index + 3) == Some(&[delimiter; 3]);
    let delimiter_width = if triple { 3 } else { 1 };
    let mut index = quote_index + delimiter_width;

    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
            continue;
        }
        if triple {
            if bytes.get(index..index + 3) == Some(&[delimiter; 3]) {
                return Some(index + 3);
            }
        } else {
            if bytes[index] == delimiter {
                return Some(index + 1);
            }
            if matches!(bytes[index], b'\r' | b'\n') {
                return None;
            }
        }
        index += 1;
    }

    None
}

fn is_python_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn javascript_comment_ranges(text: &str) -> Vec<TextRange> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    scan_javascript_code(bytes, &mut index, None, &mut ranges);
    ranges
}

struct JavascriptLexicalState {
    regex_allowed: bool,
    pending_control_parenthesis: bool,
    parenthesis_regex_after_close: Vec<bool>,
}

impl JavascriptLexicalState {
    fn new() -> Self {
        Self {
            regex_allowed: true,
            pending_control_parenthesis: false,
            parenthesis_regex_after_close: Vec::new(),
        }
    }

    fn expect_expression(&mut self) {
        self.regex_allowed = true;
        self.pending_control_parenthesis = false;
    }

    fn finish_value(&mut self) {
        self.regex_allowed = false;
        self.pending_control_parenthesis = false;
    }

    fn open_parenthesis(&mut self) {
        self.parenthesis_regex_after_close
            .push(self.pending_control_parenthesis);
        self.expect_expression();
    }

    fn close_parenthesis(&mut self) {
        self.regex_allowed = self.parenthesis_regex_after_close.pop().unwrap_or(false);
        self.pending_control_parenthesis = false;
    }

    fn finish_identifier(&mut self, token: &str) {
        self.regex_allowed = javascript_keyword_expects_expression(token);
        self.pending_control_parenthesis = javascript_control_keyword(token);
    }
}

/// Scans JavaScript code. `brace_depth` is present only inside a template interpolation.
fn scan_javascript_code(
    bytes: &[u8],
    index: &mut usize,
    mut brace_depth: Option<usize>,
    ranges: &mut Vec<TextRange>,
) -> bool {
    let mut state = JavascriptLexicalState::new();
    while *index < bytes.len() {
        match bytes[*index] {
            quote @ (b'\'' | b'"') => {
                let Some(end) = skip_escaped_quoted(bytes, *index, quote, true) else {
                    *index = bytes.len();
                    return false;
                };
                *index = end;
                state.finish_value();
            }
            b'`' => {
                if !scan_javascript_template(bytes, index, ranges) {
                    return false;
                }
                state.finish_value();
            }
            b'/' if bytes.get(*index + 1) == Some(&b'/') => {
                ranges.push(take_javascript_line_comment(bytes, index));
            }
            b'/' if bytes.get(*index + 1) == Some(&b'*') => {
                ranges.push(take_javascript_block_comment(bytes, index));
            }
            b'/' if bytes.get(*index + 1) == Some(&b'=') => {
                *index += 2;
                state.expect_expression();
            }
            b'/' if state.regex_allowed => {
                let Some(end) = skip_javascript_regex(bytes, *index) else {
                    *index = bytes.len();
                    return false;
                };
                *index = end;
                state.finish_value();
            }
            b'{' if brace_depth.is_some() => {
                brace_depth = brace_depth.map(|depth| depth + 1);
                *index += 1;
                state.expect_expression();
            }
            b'}' if brace_depth.is_some() => {
                let depth = brace_depth.expect("template interpolation has a brace depth");
                *index += 1;
                if depth == 1 {
                    return true;
                }
                brace_depth = Some(depth - 1);
                state.expect_expression();
            }
            b'(' => {
                *index += 1;
                state.open_parenthesis();
            }
            b')' => {
                *index += 1;
                state.close_parenthesis();
            }
            b']' | b'.' => {
                *index += 1;
                state.finish_value();
            }
            b'+' | b'-' => {
                let operator = bytes[*index];
                let doubled = bytes.get(*index + 1) == Some(&operator);
                *index += usize::from(doubled) + 1;
                if !doubled || state.regex_allowed {
                    state.expect_expression();
                } else {
                    state.pending_control_parenthesis = false;
                }
            }
            b'/' | b'[' | b'{' | b'}' | b',' | b';' | b':' | b'?' | b'=' | b'!' | b'~' | b'*'
            | b'%' | b'&' | b'|' | b'^' | b'<' | b'>' => {
                *index += 1;
                state.expect_expression();
            }
            byte if javascript_identifier_start(byte) => {
                let token = take_javascript_identifier(bytes, index);
                state.finish_identifier(token);
            }
            byte if byte.is_ascii_digit() => {
                skip_javascript_number(bytes, index);
                state.finish_value();
            }
            _ => *index += 1,
        }
    }

    brace_depth.is_none()
}

fn take_javascript_line_comment(bytes: &[u8], index: &mut usize) -> TextRange {
    let start = *index;
    *index = line_end(bytes, start);
    TextRange { start, end: *index }
}

fn take_javascript_block_comment(bytes: &[u8], index: &mut usize) -> TextRange {
    let start = *index;
    *index += 2;
    while *index + 1 < bytes.len() && bytes.get(*index..*index + 2) != Some(b"*/") {
        *index += 1;
    }
    *index = if *index + 1 < bytes.len() {
        *index + 2
    } else {
        bytes.len()
    };
    TextRange { start, end: *index }
}

fn take_javascript_identifier<'a>(bytes: &'a [u8], index: &mut usize) -> &'a str {
    let start = *index;
    *index += 1;
    while bytes
        .get(*index)
        .is_some_and(|byte| javascript_identifier_continue(*byte))
    {
        *index += 1;
    }
    std::str::from_utf8(&bytes[start..*index]).unwrap_or_default()
}

fn skip_javascript_number(bytes: &[u8], index: &mut usize) {
    *index += 1;
    while bytes
        .get(*index)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
    {
        *index += 1;
    }
}

fn skip_javascript_regex(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 1;
    let mut in_character_class = false;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if bytes
                    .get(index + 1)
                    .is_none_or(|byte| matches!(byte, b'\r' | b'\n'))
                {
                    return None;
                }
                index += 2;
            }
            b'[' if !in_character_class => {
                in_character_class = true;
                index += 1;
            }
            b']' if in_character_class => {
                in_character_class = false;
                index += 1;
            }
            b'/' if !in_character_class => {
                index += 1;
                while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    index += 1;
                }
                return Some(index);
            }
            b'\r' | b'\n' => return None,
            _ => index += 1,
        }
    }

    None
}

fn javascript_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
}

fn javascript_identifier_continue(byte: u8) -> bool {
    javascript_identifier_start(byte) || byte.is_ascii_digit()
}

fn javascript_keyword_expects_expression(token: &str) -> bool {
    matches!(
        token,
        "await"
            | "case"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "extends"
            | "in"
            | "instanceof"
            | "new"
            | "of"
            | "return"
            | "throw"
            | "typeof"
            | "void"
            | "yield"
    )
}

fn javascript_control_keyword(token: &str) -> bool {
    matches!(token, "catch" | "for" | "if" | "switch" | "while" | "with")
}

fn scan_javascript_template(bytes: &[u8], index: &mut usize, ranges: &mut Vec<TextRange>) -> bool {
    *index += 1;
    while *index < bytes.len() {
        match bytes[*index] {
            b'\\' => *index = (*index + 2).min(bytes.len()),
            b'`' => {
                *index += 1;
                return true;
            }
            b'$' if bytes.get(*index + 1) == Some(&b'{') => {
                *index += 2;
                if !scan_javascript_code(bytes, index, Some(1), ranges) {
                    return false;
                }
            }
            _ => *index += 1,
        }
    }
    false
}

fn rust_comment_ranges(text: &str) -> Vec<TextRange> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if let Some((content_start, hashes)) = rust_raw_string_start(bytes, index) {
            let Some(end) = skip_rust_raw_string(bytes, content_start, hashes) else {
                break;
            };
            index = end;
            continue;
        }

        match bytes[index] {
            b'"' => {
                let Some(end) = skip_escaped_quoted(bytes, index, b'"', false) else {
                    break;
                };
                index = end;
            }
            b'\'' => match skip_rust_char(bytes, index) {
                RustChar::Closed(end) => index = end,
                RustChar::Lifetime => index += 1,
                RustChar::Unclosed => break,
            },
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let start = index;
                let end = line_end(bytes, start);
                push_non_empty_range(&mut ranges, start, end);
                index = end;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let start = index;
                index += 2;
                let mut depth = 1_usize;
                while index < bytes.len() && depth > 0 {
                    if bytes.get(index..index + 2) == Some(b"/*") {
                        depth += 1;
                        index += 2;
                    } else if bytes.get(index..index + 2) == Some(b"*/") {
                        depth -= 1;
                        index += 2;
                    } else {
                        index += 1;
                    }
                }
                push_non_empty_range(&mut ranges, start, index);
            }
            _ => index += 1,
        }
    }

    ranges
}

fn rust_raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let mut cursor = index;
    if bytes.get(cursor) == Some(&b'b') {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'r') {
        return None;
    }
    cursor += 1;
    let hash_start = cursor;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor + 1, cursor - hash_start))
}

fn skip_rust_raw_string(bytes: &[u8], mut index: usize, hashes: usize) -> Option<usize> {
    while index < bytes.len() {
        let hash_end = index + 1 + hashes;
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..hash_end)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(hash_end);
        }
        index += 1;
    }
    None
}

enum RustChar {
    Closed(usize),
    Lifetime,
    Unclosed,
}

fn skip_rust_char(bytes: &[u8], index: usize) -> RustChar {
    let Some(&first) = bytes.get(index + 1) else {
        return RustChar::Unclosed;
    };
    if first.is_ascii_alphabetic() || first == b'_' {
        let character_end = index + 2;
        return if bytes.get(character_end) == Some(&b'\'') {
            RustChar::Closed(character_end + 1)
        } else {
            RustChar::Lifetime
        };
    }

    let character_end = if first == b'\\' {
        rust_escape_end(bytes, index + 1)
    } else {
        next_utf8_boundary(bytes, index + 1)
    };
    match character_end {
        Some(end) if bytes.get(end) == Some(&b'\'') => RustChar::Closed(end + 1),
        _ => RustChar::Unclosed,
    }
}

fn rust_escape_end(bytes: &[u8], backslash: usize) -> Option<usize> {
    let escape = *bytes.get(backslash + 1)?;
    if escape == b'u' && bytes.get(backslash + 2) == Some(&b'{') {
        let close = bytes[backslash + 3..]
            .iter()
            .position(|byte| *byte == b'}')?;
        Some(backslash + 3 + close + 1)
    } else if escape == b'x' {
        (backslash + 4 <= bytes.len()).then_some(backslash + 4)
    } else {
        Some(backslash + 2)
    }
}

fn next_utf8_boundary(bytes: &[u8], index: usize) -> Option<usize> {
    let width = match *bytes.get(index)? {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => return None,
    };
    (index + width <= bytes.len()).then_some(index + width)
}

fn skip_escaped_quoted(
    bytes: &[u8],
    start: usize,
    delimiter: u8,
    newline_is_unclosed: bool,
) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == delimiter {
            return Some(index + 1);
        } else if newline_is_unclosed && matches!(bytes[index], b'\r' | b'\n') {
            return None;
        } else {
            index += 1;
        }
    }
    None
}

fn line_end(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .position(|byte| matches!(byte, b'\r' | b'\n'))
        .map_or(bytes.len(), |relative_end| start + relative_end)
}

fn push_non_empty_range(ranges: &mut Vec<TextRange>, start: usize, end: usize) {
    if start < end {
        ranges.push(TextRange { start, end });
    }
}
