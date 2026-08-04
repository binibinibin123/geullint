#![allow(clippy::cast_possible_truncation)]

/// Hangul syllable decomposition used by spelling candidate generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyllableFeatures {
    pub initial: u8,
    pub medial: u8,
    pub final_consonant: u8,
}

#[must_use]
pub fn decompose_syllable(character: char) -> Option<SyllableFeatures> {
    let code = character as u32;
    if !(0xAC00..=0xD7A3).contains(&code) {
        return None;
    }
    let offset = code - 0xAC00;
    Some(SyllableFeatures {
        initial: u8::try_from(offset / 588).unwrap_or_default(),
        medial: u8::try_from((offset % 588) / 28).unwrap_or_default(),
        final_consonant: u8::try_from(offset % 28).unwrap_or_default(),
    })
}

#[must_use]
pub fn compose_syllable(features: SyllableFeatures) -> Option<char> {
    if features.initial >= 19 || features.medial >= 21 || features.final_consonant >= 28 {
        return None;
    }
    char::from_u32(
        0xAC00
            + u32::from(features.initial) * 588
            + u32::from(features.medial) * 28
            + u32::from(features.final_consonant),
    )
}

#[must_use]
pub fn phonology_distance(left: char, right: char) -> u8 {
    let Some(left) = decompose_syllable(left) else {
        return u8::from(left != right);
    };
    let Some(right) = decompose_syllable(right) else {
        return 1;
    };
    u8::from(left.initial != right.initial)
        + u8::from(left.medial != right.medial)
        + u8::from(left.final_consonant != right.final_consonant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_modern_hangul_syllables() {
        for character in ['가', '한', '힣'] {
            assert_eq!(
                compose_syllable(decompose_syllable(character).unwrap()),
                Some(character)
            );
        }
    }

    #[test]
    fn keeps_phonology_distance_bounded_and_non_hangul_safe() {
        assert_eq!(phonology_distance('가', '가'), 0);
        assert!(phonology_distance('가', '나') <= 3);
        assert_eq!(phonology_distance('가', 'a'), 1);
    }
}
