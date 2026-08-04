#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::Confidence;
use serde::Deserialize;

const HASH_DIM: usize = 256;
const FEATURE_DIM: usize = HASH_DIM + 4;

#[derive(Clone, Debug)]
pub struct ContextRanker {
    feature_scale: f32,
    weight_scale: f32,
    weights: Vec<i8>,
    bias: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContextRankerManifest {
    schema_version: u8,
    format: String,
    feature_dim: usize,
    feature_scale: f32,
    weight_scale: f32,
    weights: Vec<i8>,
    bias: f32,
}

impl ContextRanker {
    /// Parse the dependency-free quantized context model used by native and WASM.
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON is malformed or does not match the supported model
    /// schema.
    pub fn from_manifest_str(source: &str) -> Result<Self, String> {
        let manifest: ContextRankerManifest =
            serde_json::from_str(source).map_err(|error| error.to_string())?;
        if manifest.schema_version != 1
            || manifest.format != "geulrank-context-linear-int8-v1"
            || manifest.feature_dim != FEATURE_DIM
            || manifest.weights.len() != FEATURE_DIM
            || !manifest.feature_scale.is_finite()
            || manifest.feature_scale <= 0.0
            || !manifest.weight_scale.is_finite()
            || manifest.weight_scale <= 0.0
            || !manifest.bias.is_finite()
        {
            return Err("unsupported context ranker manifest".to_owned());
        }
        Ok(Self {
            feature_scale: manifest.feature_scale,
            weight_scale: manifest.weight_scale,
            weights: manifest.weights,
            bias: manifest.bias,
        })
    }

    /// Load the checked-in experimental context model.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundled model fails schema validation.
    #[cfg(feature = "standard")]
    pub fn bundled() -> Result<Self, String> {
        Self::from_manifest_str(include_str!(
            "../../../models/geulrank-small/context-ranker/context-linear-int8.json"
        ))
    }

    #[must_use]
    pub fn score(&self, source: &str, candidate: &str) -> f32 {
        let features = feature_vector(source, candidate);
        let mut dot_product = 0_i32;
        for (feature, weight) in features.into_iter().zip(&self.weights) {
            let quantized = (feature / self.feature_scale).round().clamp(-127.0, 127.0) as i32;
            dot_product += quantized * i32::from(*weight);
        }
        dot_product as f32 * self.feature_scale * self.weight_scale + self.bias
    }

    #[must_use]
    pub fn confidence(&self, score: f32) -> Confidence {
        let probability = 1.0 / (1.0 + (-score).exp());
        if probability >= 0.85 {
            Confidence::High
        } else if probability >= 0.6 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }
}

fn feature_vector(source: &str, candidate: &str) -> Vec<f32> {
    let mut features = hashed_context(source, candidate);
    let source_chars: Vec<char> = source.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    let common = source_chars
        .iter()
        .zip(&candidate_chars)
        .take_while(|(left, right)| left == right)
        .count();
    let length = source_chars.len().max(candidate_chars.len()).max(1) as f32;
    features.extend([
        1.0,
        1.0 - common as f32 / length,
        (candidate_chars.len() as f32 + 1.0).ln(),
        candidate_chars.len() as f32 - source_chars.len() as f32,
    ]);
    debug_assert_eq!(features.len(), FEATURE_DIM);
    features
}

fn hashed_context(source: &str, candidate: &str) -> Vec<f32> {
    let mut values = vec![0.0_f32; HASH_DIM];
    for (segment, marker) in [(source, 'S'), (candidate, 'C')] {
        let normalized = segment.split_whitespace().collect::<Vec<_>>().join(" ");
        let chars: Vec<char> = normalized.chars().collect();
        for size in 1..=3 {
            if chars.len() < size {
                continue;
            }
            for index in 0..=chars.len() - size {
                let ngram: String = chars[index..index + size].iter().collect();
                let bucket = fnv_bucket(&format!("{marker}:{ngram}"), size);
                values[bucket] += 1.0;
            }
        }
    }
    let normalizer = values.iter().sum::<f32>().max(1.0);
    for value in &mut values {
        *value /= normalizer;
    }
    values
}

fn fnv_bucket(value: &str, salt: usize) -> usize {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in format!("{salt}:{value}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    usize::try_from(hash % HASH_DIM as u64).expect("context hash bucket fits usize")
}

#[cfg(test)]
mod tests {
    use super::{ContextRanker, FEATURE_DIM};

    #[test]
    fn bundled_context_ranker_is_valid_and_deterministic() {
        let ranker = ContextRanker::bundled().expect("context ranker");
        let first = ranker.score("몇일 뒤에 만나요.", "며칠 뒤에 만나요.");
        let second = ranker.score("몇일 뒤에 만나요.", "며칠 뒤에 만나요.");
        assert!((first - second).abs() < f32::EPSILON);
        assert!(first.is_finite());
        assert_eq!(FEATURE_DIM, 260);
    }
}
