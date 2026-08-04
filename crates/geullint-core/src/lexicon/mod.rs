use std::collections::BTreeMap;

/// One deterministic standard-lexicon entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexiconEntry {
    pub surface: String,
    pub part_of_speech: String,
    pub frequency: u64,
}

/// Errors raised when a versioned standard lexicon is malformed.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum LexiconError {
    #[error("standard lexicon must start with geullint-standard-lexicon-v1")]
    MissingVersionHeader,
    #[error("standard lexicon row {line} is malformed")]
    InvalidRow { line: usize },
    #[error("standard lexicon row {line} is not sorted")]
    Unsorted { line: usize },
    #[error("standard lexicon contains duplicate surface `{surface}`")]
    Duplicate { surface: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StandardLexicon {
    entries: BTreeMap<String, LexiconEntry>,
}

impl StandardLexicon {
    /// Parse the deterministic tab-separated v1 interchange format.
    pub fn parse(source: &str) -> Result<Self, LexiconError> {
        let mut lines = source.lines();
        if lines.next() != Some("geullint-standard-lexicon-v1") {
            return Err(LexiconError::MissingVersionHeader);
        }
        let mut entries = BTreeMap::new();
        let mut previous_surface: Option<String> = None;
        for (index, line) in lines.enumerate() {
            let line_number = index + 2;
            if line.is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 3
                || fields[0].is_empty()
                || fields[1].is_empty()
                || fields[2].parse::<u64>().is_err()
            {
                return Err(LexiconError::InvalidRow { line: line_number });
            }
            let surface = fields[0].to_owned();
            if previous_surface
                .as_deref()
                .is_some_and(|previous| surface.as_str() <= previous)
            {
                if entries.contains_key(&surface) {
                    return Err(LexiconError::Duplicate { surface });
                }
                return Err(LexiconError::Unsorted { line: line_number });
            }
            previous_surface = Some(surface.clone());
            entries.insert(
                surface.clone(),
                LexiconEntry {
                    surface,
                    part_of_speech: fields[1].to_owned(),
                    frequency: fields[2]
                        .parse()
                        .expect("frequency was validated as an unsigned integer"),
                },
            );
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn lookup(&self, surface: &str) -> Option<&LexiconEntry> {
        self.entries.get(surface)
    }

    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = &LexiconEntry> {
        self.entries.values()
    }
}
