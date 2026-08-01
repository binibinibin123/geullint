#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SurfaceEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

const DWAE_TO_DOE_SUFFIXES: &[&str] = &["게", "게요", "면", "고", "는", "겠", "면서", "도록"];

const DOE_TO_DWAE_FORMS: &[(&str, &str)] = &[
    ("되서", "돼서"),
    ("되요", "돼요"),
    ("되야", "돼야"),
    ("됀", "된"),
    ("됄", "될"),
    ("됌", "됨"),
];

const KNOWN_EURYEO_FORMS: &[(&str, &str)] = &[
    ("할려고", "하려고"),
    ("할려면", "하려면"),
    ("먹을려고", "먹으려고"),
    ("먹을려면", "먹으려면"),
    ("읽을려고", "읽으려고"),
    ("읽을려면", "읽으려면"),
    ("잡을려고", "잡으려고"),
    ("잡을려면", "잡으려면"),
];

const CONTEXT_EURYEO_FORMS: &[(&str, &str)] = &[("갈려고", "가려고"), ("갈려면", "가려면")];

pub(crate) fn correct_dwae_to_doe(word: &str) -> Option<String> {
    let (start, _) = word.match_indices('돼').find(|(start, matched)| {
        let suffix = &word[*start + matched.len()..];
        DWAE_TO_DOE_SUFFIXES.contains(&suffix)
    })?;
    let mut corrected = word.to_owned();
    corrected.replace_range(start..start + '돼'.len_utf8(), "되");
    Some(corrected)
}

pub(crate) fn doe_to_dwae_edits(word: &str) -> Vec<SurfaceEdit> {
    let mut edits = matching_edits(word, DOE_TO_DWAE_FORMS);
    if word == "되도" {
        edits.push(SurfaceEdit {
            start: 0,
            end: word.len(),
            replacement: "돼도".to_owned(),
        });
    }
    edits.sort_by_key(|edit| (edit.start, edit.end));
    edits.dedup_by(|right, left| right.start == left.start && right.end == left.end);
    edits
}

pub(crate) fn correct_known_euryeo(word: &str) -> Option<String> {
    replace_first_form(word, KNOWN_EURYEO_FORMS)
}

pub(crate) fn review_context_euryeo(word: &str) -> Option<String> {
    replace_first_form(word, CONTEXT_EURYEO_FORMS)
}

pub(crate) fn review_colloquial_yong(word: &str) -> Option<String> {
    if let Some(stem) = word.strip_suffix("해용") {
        return Some(format!("{stem}해요"));
    }
    word.strip_suffix("세용").map(|stem| format!("{stem}세요"))
}

fn matching_edits(word: &str, forms: &[(&str, &str)]) -> Vec<SurfaceEdit> {
    forms
        .iter()
        .flat_map(|(incorrect, correct)| {
            word.match_indices(incorrect)
                .map(move |(start, matched)| SurfaceEdit {
                    start,
                    end: start + matched.len(),
                    replacement: (*correct).to_owned(),
                })
        })
        .collect()
}

fn replace_first_form(word: &str, forms: &[(&str, &str)]) -> Option<String> {
    let (start, incorrect, correct) = forms
        .iter()
        .filter_map(|(incorrect, correct)| {
            word.find(incorrect)
                .map(|start| (start, *incorrect, *correct))
        })
        .min_by_key(|(start, incorrect, _)| (*start, usize::MAX - incorrect.len()))?;
    let mut corrected = word.to_owned();
    corrected.replace_range(start..start + incorrect.len(), correct);
    Some(corrected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audited_dwae_suffixes_are_explicit_and_controls_stay_quiet() {
        assert_eq!(correct_dwae_to_doe("안돼게요").as_deref(), Some("안되게요"));
        assert_eq!(
            correct_dwae_to_doe("진행돼면서").as_deref(),
            Some("진행되면서")
        );
        assert_eq!(
            correct_dwae_to_doe("적용돼도록").as_deref(),
            Some("적용되도록")
        );
        for control in [
            "돼서",
            "돼도",
            "돼요",
            "돼야",
            "돼지",
            "돼지만",
            "돼지도",
            "돼고기",
        ] {
            assert_eq!(correct_dwae_to_doe(control), None, "{control}");
        }
    }

    #[test]
    fn known_euryeo_forms_preserve_prefixes_and_suffixes() {
        assert_eq!(
            correct_known_euryeo("재확인할려고도").as_deref(),
            Some("재확인하려고도")
        );
        assert_eq!(
            correct_known_euryeo("먹을려면").as_deref(),
            Some("먹으려면")
        );
        assert_eq!(correct_known_euryeo("좋을려고"), None);
    }

    #[test]
    fn doe_edits_keep_byte_ranges_local_to_the_word() {
        let word = "안됀다면됄까요됌을";
        let edits = doe_to_dwae_edits(word);

        assert_eq!(edits.len(), 3);
        assert_eq!(
            edits
                .iter()
                .map(|edit| (&word[edit.start..edit.end], edit.replacement.as_str()))
                .collect::<Vec<_>>(),
            [("됀", "된"), ("됄", "될"), ("됌", "됨")]
        );
    }

    #[test]
    fn do_form_requires_the_whole_eojeol() {
        assert_eq!(
            doe_to_dwae_edits("되도"),
            [SurfaceEdit {
                start: 0,
                end: "되도".len(),
                replacement: "돼도".to_owned(),
            }]
        );
        for control in ["되도록", "되돌리다", "되돌아가다"] {
            assert!(doe_to_dwae_edits(control).is_empty(), "{control}");
        }
    }

    #[test]
    fn context_and_colloquial_rules_use_small_audited_surfaces() {
        assert_eq!(review_context_euryeo("갈려고").as_deref(), Some("가려고"));
        for control in ["알려고", "밀려고", "들려고", "그을려고"] {
            assert_eq!(review_context_euryeo(control), None, "{control}");
        }
        assert_eq!(
            review_colloquial_yong("감사해용").as_deref(),
            Some("감사해요")
        );
        assert_eq!(review_colloquial_yong("오세용").as_deref(), Some("오세요"));
        for control in ["사용", "내용", "허용", "군용", "어용", "지용"] {
            assert_eq!(review_colloquial_yong(control), None, "{control}");
        }
    }
}
