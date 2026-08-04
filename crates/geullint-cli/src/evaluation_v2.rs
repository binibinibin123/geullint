use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusOrigin {
    IndependentHuman,
    Revision,
    Project,
    Synthetic,
}

impl Default for CorpusOrigin {
    fn default() -> Self {
        Self::Project
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusSplit {
    Train,
    Dev,
    ReleaseHoldout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseMetadata {
    pub id: String,
    pub origin: CorpusOrigin,
    pub split: Option<CorpusSplit>,
    pub genre: Option<String>,
    pub document_id: Option<String>,
    pub normal: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DatasetQualityGate {
    #[serde(default)]
    pub min_cases: Option<usize>,
    #[serde(default)]
    pub min_natural_cases: Option<usize>,
    #[serde(default)]
    pub min_human_edit_cases: Option<usize>,
    #[serde(default)]
    pub min_normal_cases: Option<usize>,
    #[serde(default)]
    pub min_genres: Option<usize>,
    #[serde(default)]
    pub min_documents: Option<usize>,
    #[serde(default)]
    pub require_release_holdout: bool,
    #[serde(default)]
    pub require_independent_human: bool,
    #[serde(default)]
    pub reject_synthetic: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetMetadata {
    pub cases: usize,
    pub natural_cases: usize,
    pub human_edit_cases: usize,
    pub normal_cases: usize,
    pub synthetic_cases: usize,
    pub independent_human_cases: usize,
    pub genres: Vec<String>,
    pub documents: usize,
    pub splits: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DatasetGateFailure {
    pub metric: &'static str,
    pub actual: usize,
    pub minimum: usize,
}

pub(crate) fn aggregate_metadata(cases: &[CaseMetadata]) -> DatasetMetadata {
    let mut metadata = DatasetMetadata {
        cases: cases.len(),
        ..DatasetMetadata::default()
    };
    let mut genres = BTreeSet::new();
    let mut documents = BTreeSet::new();

    for case in cases {
        if case.origin == CorpusOrigin::Synthetic {
            metadata.synthetic_cases += 1;
        } else {
            metadata.natural_cases += 1;
        }
        if !case.normal
            && matches!(
                case.origin,
                CorpusOrigin::IndependentHuman | CorpusOrigin::Revision
            )
        {
            metadata.human_edit_cases += 1;
        }
        if case.normal {
            metadata.normal_cases += 1;
        }
        if case.origin == CorpusOrigin::IndependentHuman {
            metadata.independent_human_cases += 1;
        }
        if let Some(genre) = case
            .genre
            .as_deref()
            .filter(|genre| !genre.trim().is_empty())
        {
            genres.insert(genre.trim().to_owned());
        }
        if let Some(document_id) = case
            .document_id
            .as_deref()
            .filter(|document_id| !document_id.trim().is_empty())
        {
            documents.insert(document_id.trim().to_owned());
        }
        if let Some(split) = case.split {
            *metadata
                .splits
                .entry(split.as_str().to_owned())
                .or_default() += 1;
        }
    }

    metadata.genres = genres.into_iter().collect();
    metadata.documents = documents.len();
    metadata
}

pub(crate) fn evaluate_dataset_gate(
    metadata: &DatasetMetadata,
    gate: &DatasetQualityGate,
) -> Vec<DatasetGateFailure> {
    let mut failures = Vec::new();
    push_minimum(&mut failures, "cases", metadata.cases, gate.min_cases);
    push_minimum(
        &mut failures,
        "naturalCases",
        metadata.natural_cases,
        gate.min_natural_cases,
    );
    push_minimum(
        &mut failures,
        "humanEditCases",
        metadata.human_edit_cases,
        gate.min_human_edit_cases,
    );
    push_minimum(
        &mut failures,
        "normalCases",
        metadata.normal_cases,
        gate.min_normal_cases,
    );
    push_minimum(
        &mut failures,
        "genres",
        metadata.genres.len(),
        gate.min_genres,
    );
    push_minimum(
        &mut failures,
        "documents",
        metadata.documents,
        gate.min_documents,
    );
    if gate.require_release_holdout {
        let actual = metadata.splits.get("release_holdout").copied().unwrap_or(0);
        if actual == 0 {
            failures.push(DatasetGateFailure {
                metric: "releaseHoldout",
                actual,
                minimum: 1,
            });
        }
    }
    if gate.require_independent_human && metadata.independent_human_cases == 0 {
        failures.push(DatasetGateFailure {
            metric: "independentHumanCases",
            actual: 0,
            minimum: 1,
        });
    }
    if gate.reject_synthetic && metadata.synthetic_cases > 0 {
        failures.push(DatasetGateFailure {
            metric: "syntheticCases",
            actual: metadata.synthetic_cases,
            minimum: 0,
        });
    }
    failures
}

fn push_minimum(
    failures: &mut Vec<DatasetGateFailure>,
    metric: &'static str,
    actual: usize,
    minimum: Option<usize>,
) {
    if let Some(minimum) = minimum {
        if actual < minimum {
            failures.push(DatasetGateFailure {
                metric,
                actual,
                minimum,
            });
        }
    }
}

impl CorpusSplit {
    fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Dev => "dev",
            Self::ReleaseHoldout => "release_holdout",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(
        id: &str,
        origin: CorpusOrigin,
        split: CorpusSplit,
        genre: &str,
        document_id: &str,
        normal: bool,
    ) -> CaseMetadata {
        CaseMetadata {
            id: id.to_owned(),
            origin,
            split: Some(split),
            genre: Some(genre.to_owned()),
            document_id: Some(document_id.to_owned()),
            normal,
        }
    }

    #[test]
    fn aggregates_human_normal_synthetic_and_split_metadata() {
        let metadata = aggregate_metadata(&[
            case(
                "human-error",
                CorpusOrigin::IndependentHuman,
                CorpusSplit::ReleaseHoldout,
                "news",
                "doc-a",
                false,
            ),
            case(
                "human-normal",
                CorpusOrigin::IndependentHuman,
                CorpusSplit::ReleaseHoldout,
                "news",
                "doc-a",
                true,
            ),
            case(
                "revision-error",
                CorpusOrigin::Revision,
                CorpusSplit::Dev,
                "technical",
                "doc-b",
                false,
            ),
            case(
                "synthetic-error",
                CorpusOrigin::Synthetic,
                CorpusSplit::Train,
                "synthetic",
                "doc-c",
                false,
            ),
        ]);

        assert_eq!(metadata.cases, 4);
        assert_eq!(metadata.natural_cases, 3);
        assert_eq!(metadata.human_edit_cases, 2);
        assert_eq!(metadata.normal_cases, 1);
        assert_eq!(metadata.synthetic_cases, 1);
        assert_eq!(metadata.independent_human_cases, 2);
        assert_eq!(metadata.genres, ["news", "synthetic", "technical"]);
        assert_eq!(metadata.documents, 3);
        assert_eq!(metadata.splits["release_holdout"], 2);
    }

    #[test]
    fn rejects_a_dataset_that_misses_release_and_clean_data_requirements() {
        let metadata = aggregate_metadata(&[case(
            "human-error",
            CorpusOrigin::IndependentHuman,
            CorpusSplit::Dev,
            "news",
            "doc-a",
            false,
        )]);
        let failures = evaluate_dataset_gate(
            &metadata,
            &DatasetQualityGate {
                min_cases: Some(2),
                min_natural_cases: Some(2),
                min_human_edit_cases: Some(2),
                min_normal_cases: Some(1),
                min_genres: Some(2),
                min_documents: Some(2),
                require_release_holdout: true,
                require_independent_human: true,
                reject_synthetic: true,
            },
        );

        let metrics: Vec<_> = failures.iter().map(|failure| failure.metric).collect();
        assert!(metrics.contains(&"cases"));
        assert!(metrics.contains(&"naturalCases"));
        assert!(metrics.contains(&"humanEditCases"));
        assert!(metrics.contains(&"normalCases"));
        assert!(metrics.contains(&"genres"));
        assert!(metrics.contains(&"documents"));
        assert!(metrics.contains(&"releaseHoldout"));
    }

    #[test]
    fn rejects_synthetic_cases_when_the_gate_disallows_them() {
        let metadata = aggregate_metadata(&[case(
            "synthetic",
            CorpusOrigin::Synthetic,
            CorpusSplit::Train,
            "synthetic",
            "doc-a",
            false,
        )]);
        let failures = evaluate_dataset_gate(
            &metadata,
            &DatasetQualityGate {
                reject_synthetic: true,
                ..DatasetQualityGate::default()
            },
        );
        assert_eq!(
            failures,
            [DatasetGateFailure {
                metric: "syntheticCases",
                actual: 1,
                minimum: 0,
            }]
        );
    }
}
