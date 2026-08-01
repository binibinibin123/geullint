use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use serde::Deserialize;

use crate::{LiteralRule, Replacement};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MatchBoundary {
    Substring,
    Word,
    Prefix,
    Suffix,
}

impl MatchBoundary {
    pub(crate) fn allows(self, text: &str, start: usize, end: usize) -> bool {
        let matched = &text[start..end];
        let needs_left_boundary = matched.chars().next().is_some_and(is_word_character);
        let needs_right_boundary = matched.chars().next_back().is_some_and(is_word_character);
        let has_left_boundary = !needs_left_boundary
            || text[..start]
                .chars()
                .next_back()
                .is_none_or(|character| !is_word_character(character));
        let has_right_boundary = !needs_right_boundary
            || text[end..]
                .chars()
                .next()
                .is_none_or(|character| !is_word_character(character));

        match self {
            Self::Substring => true,
            Self::Word => has_left_boundary && has_right_boundary,
            Self::Prefix => has_left_boundary,
            Self::Suffix => has_right_boundary,
        }
    }
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LiteralMatch {
    pub(crate) rule_index: usize,
    pub(crate) replacement_index: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug)]
struct PatternReference {
    rule_index: usize,
    replacement_index: usize,
    boundary: MatchBoundary,
}

#[derive(Clone, Debug)]
pub(crate) struct LiteralMatcher {
    automaton: AhoCorasick,
    pattern_references: Vec<PatternReference>,
}

impl LiteralMatcher {
    pub(crate) fn from_bundled_rules(rules: &[LiteralRule]) -> Self {
        let patterns = rules
            .iter()
            .enumerate()
            .flat_map(|(rule_index, rule)| {
                rule.replacements
                    .iter()
                    .enumerate()
                    .map(move |(replacement_index, replacement)| {
                        (
                            replacement.from.clone(),
                            PatternReference {
                                rule_index,
                                replacement_index,
                                boundary: bundled_boundary(rule, replacement),
                            },
                        )
                    })
            })
            .collect();
        Self::build(patterns)
    }

    fn build(patterns: Vec<(String, PatternReference)>) -> Self {
        let (patterns, pattern_references): (Vec<_>, Vec<_>) = patterns.into_iter().unzip();
        let automaton = AhoCorasickBuilder::new()
            .match_kind(MatchKind::Standard)
            .build(&patterns)
            .expect("validated literal patterns must build an Aho-Corasick automaton");
        Self {
            automaton,
            pattern_references,
        }
    }

    pub(crate) fn find(&self, text: &str) -> Vec<LiteralMatch> {
        let mut matches = Vec::new();
        let mut next_start_by_pattern = vec![0; self.pattern_references.len()];
        for matched in self.automaton.find_overlapping_iter(text) {
            let pattern_index = matched.pattern().as_usize();
            let reference = self.pattern_references[pattern_index];
            if matched.start() < next_start_by_pattern[pattern_index]
                || !reference
                    .boundary
                    .allows(text, matched.start(), matched.end())
            {
                continue;
            }
            next_start_by_pattern[pattern_index] = matched.end();
            matches.push(LiteralMatch {
                rule_index: reference.rule_index,
                replacement_index: reference.replacement_index,
                start: matched.start(),
                end: matched.end(),
            });
        }
        matches.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then_with(|| left.end.cmp(&right.end))
                .then_with(|| left.rule_index.cmp(&right.rule_index))
                .then_with(|| left.replacement_index.cmp(&right.replacement_index))
        });
        matches
    }
}

fn bundled_boundary(rule: &LiteralRule, replacement: &Replacement) -> MatchBoundary {
    if let Some(boundary) = replacement.boundary {
        return boundary;
    }

    inferred_bundled_boundary(&rule.id)
}

fn inferred_bundled_boundary(rule_id: &str) -> MatchBoundary {
    if rule_id.starts_with("spelling.conjugation.")
        || rule_id.starts_with("grammar.ending.")
        || rule_id.starts_with("spacing.")
        || rule_id.starts_with("punctuation.")
    {
        // Conjugation, ending, and spacing forms are registered fragments:
        // `됬` in `안됬다`, `할께` in `말할께요`, and `뿐만아니라` in
        // `그것뿐만아니라` must remain visible inside an eojeol. Punctuation
        // likewise has no lexical edge to protect.
        MatchBoundary::Substring
    } else {
        // Lexical, confusable, style, and terminology forms normally begin an
        // eojeol. Requiring that left edge prevents compound-word collisions
        // such as `금새` inside `황금새우` while retaining following particles.
        MatchBoundary::Prefix
    }
}

#[cfg(test)]
mod tests {
    use super::{LiteralMatcher, MatchBoundary, PatternReference};

    fn matcher(patterns: &[(&str, MatchBoundary)]) -> LiteralMatcher {
        LiteralMatcher::build(
            patterns
                .iter()
                .enumerate()
                .map(|(rule_index, (pattern, boundary))| {
                    (
                        (*pattern).to_owned(),
                        PatternReference {
                            rule_index,
                            replacement_index: 0,
                            boundary: *boundary,
                        },
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn finds_overlapping_patterns_in_stable_source_order() {
        let matcher = matcher(&[
            ("가나다", MatchBoundary::Substring),
            ("나다", MatchBoundary::Substring),
        ]);

        let found = matcher.find("앞 가나다 뒤");

        assert_eq!(
            found
                .iter()
                .map(|matched| (matched.rule_index, matched.start, matched.end))
                .collect::<Vec<_>>(),
            [(0, 4, 13), (1, 7, 13)]
        );
    }

    #[test]
    fn suppresses_only_self_overlaps_of_the_same_pattern() {
        let matcher = matcher(&[(",,", MatchBoundary::Substring)]);

        let found = matcher.find(",,,");

        assert_eq!(
            found
                .iter()
                .map(|matched| (matched.rule_index, matched.start, matched.end))
                .collect::<Vec<_>>(),
            [(0, 0, 2)]
        );
    }

    #[test]
    fn applies_each_boundary_mode_on_unicode_text() {
        let matcher = matcher(&[
            ("금새", MatchBoundary::Prefix),
            ("새우", MatchBoundary::Suffix),
            ("맞춤", MatchBoundary::Word),
        ]);

        assert!(
            matcher
                .find("황금새우")
                .iter()
                .all(|matched| matched.rule_index != 0)
        );
        assert_eq!(matcher.find("금새는")[0].end, "금새".len());
        assert_eq!(matcher.find("큰새우")[0].start, "큰".len());
        assert_eq!(matcher.find("맞춤 맞춤법").len(), 1);
    }
}
