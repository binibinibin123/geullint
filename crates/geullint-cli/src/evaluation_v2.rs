use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusOrigin {
    IndependentHuman,
    Revision,
    #[default]
    Project,
    Synthetic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusTextOrigin {
    HumanAuthored,
    Revision,
    Project,
    Synthetic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusAnnotationOrigin {
    AiBlindPanel,
    HumanIndependent,
    SourceRevision,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusAnnotationStatus {
    Unreviewed,
    Reviewed,
    Adjudicated,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusSplit {
    Train,
    Dev,
    ReleaseHoldout,
    #[serde(rename = "H1")]
    H1,
    #[serde(rename = "H2")]
    H2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseMetadata {
    pub id: String,
    pub origin: CorpusOrigin,
    pub text_origin: Option<CorpusTextOrigin>,
    pub annotation_origin: Option<CorpusAnnotationOrigin>,
    pub annotation_status: Option<CorpusAnnotationStatus>,
    pub split: Option<CorpusSplit>,
    pub holdout_id: Option<String>,
    pub genre: Option<String>,
    pub document_id: Option<String>,
    pub author_id: Option<String>,
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
    pub min_authors: Option<usize>,
    #[serde(default)]
    pub min_independent_human_cases: Option<usize>,
    #[serde(default)]
    pub min_holdout_cases: Option<usize>,
    #[serde(default)]
    pub required_holdout_ids: Vec<String>,
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
    pub authors: usize,
    pub splits: BTreeMap<String, usize>,
    pub holdouts: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DatasetGateFailure {
    pub metric: &'static str,
    pub actual: usize,
    pub minimum: usize,
    pub holdout_id: Option<String>,
}

pub(crate) fn aggregate_metadata(cases: &[CaseMetadata]) -> DatasetMetadata {
    let mut metadata = DatasetMetadata {
        cases: cases.len(),
        ..DatasetMetadata::default()
    };
    let mut genres = BTreeSet::new();
    let mut documents = BTreeSet::new();
    let mut authors = BTreeSet::new();

    for case in cases {
        let text_origin = case.text_origin.unwrap_or(match case.origin {
            CorpusOrigin::IndependentHuman => CorpusTextOrigin::HumanAuthored,
            CorpusOrigin::Revision => CorpusTextOrigin::Revision,
            CorpusOrigin::Project => CorpusTextOrigin::Project,
            CorpusOrigin::Synthetic => CorpusTextOrigin::Synthetic,
        });
        if text_origin == CorpusTextOrigin::Synthetic {
            metadata.synthetic_cases += 1;
        } else if text_origin != CorpusTextOrigin::Project {
            metadata.natural_cases += 1;
        }
        if !case.normal
            && matches!(
                text_origin,
                CorpusTextOrigin::HumanAuthored | CorpusTextOrigin::Revision
            )
        {
            metadata.human_edit_cases += 1;
        }
        if case.normal {
            metadata.normal_cases += 1;
        }
        if case.origin == CorpusOrigin::IndependentHuman
            || case.annotation_origin == Some(CorpusAnnotationOrigin::HumanIndependent)
        {
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
        if let Some(author_id) = case
            .author_id
            .as_deref()
            .filter(|author_id| !author_id.trim().is_empty())
        {
            authors.insert(author_id.trim().to_owned());
        }
        if let Some(split) = case.split {
            *metadata
                .splits
                .entry(split.as_str().to_owned())
                .or_default() += 1;
        }
        let holdout_id = case.holdout_id.as_deref().or_else(|| {
            matches!(case.split, Some(CorpusSplit::H1 | CorpusSplit::H2))
                .then(|| case.split.expect("matched holdout split").as_str())
        });
        if let Some(holdout_id) = holdout_id {
            *metadata.holdouts.entry(holdout_id.to_owned()).or_default() += 1;
        }
    }

    metadata.genres = genres.into_iter().collect();
    metadata.documents = documents.len();
    metadata.authors = authors.len();
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
    push_minimum(&mut failures, "authors", metadata.authors, gate.min_authors);
    push_minimum(
        &mut failures,
        "independentHumanCases",
        metadata.independent_human_cases,
        gate.min_independent_human_cases,
    );
    if gate.require_release_holdout {
        let actual = metadata.splits.get("release_holdout").copied().unwrap_or(0);
        if actual == 0 {
            failures.push(DatasetGateFailure {
                metric: "releaseHoldout",
                actual,
                minimum: 1,
                holdout_id: None,
            });
        }
    }
    if let Some(minimum) = gate.min_holdout_cases {
        for holdout_id in &gate.required_holdout_ids {
            let actual = metadata.holdouts.get(holdout_id).copied().unwrap_or(0);
            if actual < minimum {
                failures.push(DatasetGateFailure {
                    metric: "holdoutCases",
                    actual,
                    minimum,
                    holdout_id: Some(holdout_id.clone()),
                });
            }
        }
    }
    if gate.require_independent_human && metadata.independent_human_cases == 0 {
        failures.push(DatasetGateFailure {
            metric: "independentHumanCases",
            actual: 0,
            minimum: 1,
            holdout_id: None,
        });
    }
    if gate.reject_synthetic && metadata.synthetic_cases > 0 {
        failures.push(DatasetGateFailure {
            metric: "syntheticCases",
            actual: metadata.synthetic_cases,
            minimum: 0,
            holdout_id: None,
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
    if let Some(minimum) = minimum
        && actual < minimum
    {
        failures.push(DatasetGateFailure {
            metric,
            actual,
            minimum,
            holdout_id: None,
        });
    }
}

impl CorpusSplit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Dev => "dev",
            Self::ReleaseHoldout => "release_holdout",
            Self::H1 => "H1",
            Self::H2 => "H2",
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
            text_origin: None,
            annotation_origin: None,
            annotation_status: None,
            split: Some(split),
            holdout_id: None,
            genre: Some(genre.to_owned()),
            document_id: Some(document_id.to_owned()),
            author_id: None,
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
                ..DatasetQualityGate::default()
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
                holdout_id: None,
            }]
        );
    }

    #[test]
    fn ai_panel_annotations_do_not_count_as_independent_human_evidence() {
        let metadata = aggregate_metadata(&[CaseMetadata {
            id: "ai-reviewed".to_owned(),
            origin: CorpusOrigin::Project,
            text_origin: Some(CorpusTextOrigin::HumanAuthored),
            annotation_origin: Some(CorpusAnnotationOrigin::AiBlindPanel),
            annotation_status: Some(CorpusAnnotationStatus::Reviewed),
            split: Some(CorpusSplit::H1),
            holdout_id: Some("H1".to_owned()),
            genre: Some("news".to_owned()),
            document_id: Some("doc-ai".to_owned()),
            author_id: Some("author-ai".to_owned()),
            normal: false,
        }]);
        assert_eq!(metadata.natural_cases, 1);
        assert_eq!(metadata.human_edit_cases, 1);
        assert_eq!(metadata.independent_human_cases, 0);
        assert_eq!(metadata.holdouts["H1"], 1);
        let failures = evaluate_dataset_gate(
            &metadata,
            &DatasetQualityGate {
                require_independent_human: true,
                min_holdout_cases: Some(1),
                required_holdout_ids: vec!["H1".to_owned(), "H2".to_owned()],
                ..DatasetQualityGate::default()
            },
        );
        let metrics: Vec<_> = failures.iter().map(|failure| failure.metric).collect();
        assert!(metrics.contains(&"independentHumanCases"));
        assert!(failures.iter().any(|failure| {
            failure.metric == "holdoutCases" && failure.holdout_id.as_deref() == Some("H2")
        }));
    }
}
