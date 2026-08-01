#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductiveMatch {
    pub rule_id: &'static str,
    pub message: &'static str,
    pub replacement: String,
}

pub(crate) fn for_each_match(
    word: &str,
    following_text: &str,
    mut emit: impl FnMut(ProductiveMatch),
) {
    fixed_form_matches(word, &mut emit);
    append_spacing_matches(word, following_text, &mut emit);
}

fn fixed_form_matches(word: &str, emit: &mut impl FnMut(ProductiveMatch)) {
    if let Some(replacement) = correct_legacy_seumnida(word) {
        emit(productive_match(
            "grammar.ending.seumnida",
            "현대 표준어에서는 ‘-읍니다/-읍니까’ 대신 ‘-습니다/-ㅂ니다’를 씁니다.",
            replacement,
        ));
    }
    if let Some(stem) = word.strip_suffix("십시요") {
        emit(productive_match(
            "grammar.ending.sipsio",
            "높임 명령형은 ‘-십시오’로 씁니다.",
            format!("{stem}십시오"),
        ));
    }
    if word == "아니예요" {
        emit(productive_match(
            "grammar.copula.anieyo",
            "‘아니다’의 활용형은 ‘아니에요’로 씁니다.",
            "아니에요".to_owned(),
        ));
    }
    if let Some(rest) = (word != "않되")
        .then(|| word.strip_prefix('않'))
        .flatten()
        .filter(|rest| starts_with_any(rest, &["되", "돼", "된", "될", "됨", "됩", "됐"]))
    {
        emit(productive_match(
            "grammar.negation.anh-doe",
            "‘않-’이 아니라 부정 부사 ‘안’을 ‘되다’와 띄어 쓰세요.",
            format!("안 {rest}"),
        ));
    }
}

fn append_spacing_matches(
    word: &str,
    following_text: &str,
    emit: &mut impl FnMut(ProductiveMatch),
) {
    push_spacing_match(
        word,
        "데",
        "spacing.dependent-noun.de",
        "장소·경우를 나타내는 의존 명사 ‘데’는 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, suffix| {
            prefix != "쓸"
                && prefix.chars().next_back().is_some_and(has_rieul_final)
                && starts_with_any(suffix, &["가", "를", "도", "는", "에", "까지", "마다"])
        },
    );
    push_spacing_match(
        word,
        "채",
        "spacing.dependent-noun.chae",
        "이미 있는 상태를 나타내는 의존 명사 ‘채’는 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, suffix| {
            prefix != "산"
                && prefix.chars().next_back().is_some_and(has_nieun_final)
                && suffix.starts_with("로")
        },
    );
    push_spacing_match(
        word,
        "듯",
        "spacing.dependent-noun.deut",
        "짐작을 나타내는 의존 명사 ‘듯’은 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, suffix| {
            !["그럴", "번", "반"]
                .iter()
                .any(|excluded| prefix.ends_with(excluded))
                && prefix.chars().next_back().is_some_and(|character| {
                    matches!(character, '은' | '는' | '을') || has_nieun_or_rieul_final(character)
                })
                && starts_with_any(suffix, &["하", "싶", "이"])
        },
    );
    push_spacing_match(
        word,
        "만큼",
        "spacing.dependent-noun.mankeum",
        "용언을 수식하는 의존 명사 ‘만큼’은 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, _| {
            !["마을", "가을", "노을", "은"].contains(&prefix)
                && prefix
                    .chars()
                    .next_back()
                    .is_some_and(|character| matches!(character, '은' | '는' | '을'))
        },
    );
    push_spacing_match(
        word,
        "대로",
        "spacing.dependent-noun.daero",
        "용언 뒤의 의존 명사 ‘대로’는 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, _| {
            !["마을", "가을", "노을"].contains(&prefix)
                && prefix
                    .chars()
                    .next_back()
                    .is_some_and(|character| matches!(character, '은' | '는' | '을'))
        },
    );
    push_spacing_match(
        word,
        "법",
        "spacing.dependent-noun.beop",
        "일반적인 이치를 나타내는 의존 명사 ‘법’은 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, suffix| prefix.ends_with('는') && starts_with_any(suffix, &["이", "입", "도"]),
    );
    push_spacing_match(
        word,
        "리",
        "spacing.dependent-noun.ri",
        "가능성을 나타내는 의존 명사 ‘리’는 앞말과 띄어 쓰는지 검토하세요.",
        emit,
        |prefix, suffix| {
            !["물", "멀", "빨", "랠", "칼"].contains(&prefix)
                && prefix.chars().next_back().is_some_and(has_rieul_final)
                && starts_with_any(suffix, &["가", "는", "도"])
                && following_starts_with_eopda(following_text)
        },
    );
}

fn productive_match(
    rule_id: &'static str,
    message: &'static str,
    replacement: String,
) -> ProductiveMatch {
    ProductiveMatch {
        rule_id,
        message,
        replacement,
    }
}

fn correct_legacy_seumnida(word: &str) -> Option<String> {
    let (stem, tail) = word
        .strip_suffix("읍니다")
        .map(|stem| (stem, "니다"))
        .or_else(|| word.strip_suffix("읍니까").map(|stem| (stem, "니까")))?;
    let last = stem.chars().next_back()?;
    let final_index = hangul_final_index(last)?;

    if final_index == 0 || final_index == 8 {
        let changed = with_bieup_final(last);
        let stem_without_last = &stem[..stem.len() - last.len_utf8()];
        Some(format!("{stem_without_last}{changed}{tail}"))
    } else {
        Some(format!("{stem}습{tail}"))
    }
}

fn push_spacing_match(
    word: &str,
    marker: &str,
    rule_id: &'static str,
    message: &'static str,
    emit: &mut impl FnMut(ProductiveMatch),
    accepts: impl FnOnce(&str, &str) -> bool,
) {
    let Some(marker_start) = word.rfind(marker) else {
        return;
    };
    let marker_end = marker_start + marker.len();
    let prefix = &word[..marker_start];
    let suffix = &word[marker_end..];
    if prefix.is_empty() || !accepts(prefix, suffix) {
        return;
    }
    emit(productive_match(
        rule_id,
        message,
        format!("{prefix} {marker}{suffix}"),
    ));
}

fn starts_with_any(text: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| text.starts_with(prefix))
}

fn following_starts_with_eopda(text: &str) -> bool {
    let trimmed = text.trim_start_matches(char::is_whitespace);
    trimmed.len() < text.len() && trimmed.starts_with("없")
}

fn hangul_final_index(character: char) -> Option<u32> {
    ('가'..='힣')
        .contains(&character)
        .then_some((character as u32 - '가' as u32) % 28)
}

fn has_rieul_final(character: char) -> bool {
    hangul_final_index(character) == Some(8)
}

fn has_nieun_final(character: char) -> bool {
    hangul_final_index(character) == Some(4)
}

fn has_nieun_or_rieul_final(character: char) -> bool {
    matches!(hangul_final_index(character), Some(4 | 8))
}

fn with_bieup_final(character: char) -> char {
    let syllable = character as u32 - '가' as u32;
    char::from_u32('가' as u32 + (syllable / 28 * 28) + 17)
        .expect("a modern Hangul syllable with a bieup final")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_polite_endings_follow_final_consonant_allomorphy() {
        assert_eq!(
            correct_legacy_seumnida("읽읍니다").as_deref(),
            Some("읽습니다")
        );
        assert_eq!(
            correct_legacy_seumnida("쓰읍니다").as_deref(),
            Some("씁니다")
        );
        assert_eq!(
            correct_legacy_seumnida("열읍니까").as_deref(),
            Some("엽니까")
        );
        assert_eq!(correct_legacy_seumnida("먹읍시다"), None);
        assert_eq!(correct_legacy_seumnida("읍니다"), None);
    }
}
