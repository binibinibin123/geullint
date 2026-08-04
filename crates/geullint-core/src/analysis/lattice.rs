use crate::{AnalyzedDocument, AnalyzedWord, MorphToken, TextRange};

/// Immutable analysis lattice shared by candidate and ranking stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisLattice {
    pub source_ranges: Vec<TextRange>,
    pub words: Vec<AnalyzedWord>,
    pub morphology_tokens: Vec<MorphToken>,
}

impl From<&AnalyzedDocument> for AnalysisLattice {
    fn from(document: &AnalyzedDocument) -> Self {
        Self {
            source_ranges: document.source_ranges().to_vec(),
            words: document.words().to_vec(),
            morphology_tokens: document.morphology_tokens().to_vec(),
        }
    }
}
