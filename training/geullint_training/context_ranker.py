"""Deterministic hashed context features for the local learned ranker."""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Iterable


HASH_DIM = 256
NUMERIC_DIM = 4
FEATURE_DIM = HASH_DIM + NUMERIC_DIM


def _bucket(value: str, salt: int) -> int:
    # FNV-1a keeps the feature contract dependency-free and easy to reproduce
    # in Rust/WASM. Do not replace this with Python's salted ``hash()``.
    hashed = 14695981039346656037
    for byte in f"{salt}:{value}".encode("utf-8"):
        hashed ^= byte
        hashed = (hashed * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return hashed % HASH_DIM


def _hashed_context(source: str, candidate: str) -> list[float]:
    values = [0.0] * HASH_DIM
    for segment, marker in ((source, "S"), (candidate, "C")):
        normalized = " ".join(segment.split())
        codepoints = list(normalized)
        for size in (1, 2, 3):
            for index in range(max(0, len(codepoints) - size + 1)):
                ngram = "".join(codepoints[index : index + size])
                values[_bucket(f"{marker}:{ngram}", size)] += 1.0
    normalizer = max(sum(values), 1.0)
    return [value / normalizer for value in values]


def feature_vector(source: str, candidate: str) -> list[float]:
    """Return stable context and edit features for one candidate."""

    if not isinstance(source, str) or not isinstance(candidate, str):
        raise TypeError("source and candidate must be strings")
    source_chars = list(source)
    candidate_chars = list(candidate)
    common = 0
    for left, right in zip(source_chars, candidate_chars):
        if left != right:
            break
        common += 1
    length = max(len(source_chars), len(candidate_chars), 1)
    edit_ratio = 1.0 - (common / length)
    return _hashed_context(source, candidate) + [
        1.0,
        edit_ratio,
        math.log1p(len(candidate_chars)),
        float(len(candidate_chars) - len(source_chars)),
    ]


def quantize_feature_vector(values: list[float]) -> tuple[list[int], float]:
    if len(values) != FEATURE_DIM:
        raise ValueError(f"expected {FEATURE_DIM} features")
    maximum = max(abs(float(value)) for value in values)
    scale = maximum / 127.0 if maximum else 1.0
    quantized = [max(-127, min(127, round(float(value) / scale))) for value in values]
    return quantized, scale


def dot(left: list[float], right: list[float]) -> float:
    if len(left) != len(right):
        raise ValueError("feature vectors must have equal dimensions")
    return sum(a * b for a, b in zip(left, right))


def train_pairwise(
    rows: Iterable[dict],
    *,
    epochs: int = 24,
    learning_rate: float = 0.05,
    l2: float = 1e-4,
) -> dict:
    """Train a deterministic pairwise linear ranker over local context features."""

    rows = list(rows)
    if not rows:
        raise ValueError("training rows must not be empty")
    if epochs <= 0 or learning_rate <= 0 or l2 < 0:
        raise ValueError("epochs and learning_rate must be positive; l2 cannot be negative")
    differences: list[list[float]] = []
    for row in rows:
        if row.get("split") == "release_holdout":
            raise ValueError("release_holdout cannot be used for training")
        source = row.get("sourceText")
        chosen = row.get("chosenText")
        rejected = row.get("rejectedText")
        if not all(isinstance(value, str) for value in (source, chosen, rejected)):
            raise ValueError("each row requires sourceText, chosenText, and rejectedText")
        chosen_features = feature_vector(source, chosen)
        rejected_features = feature_vector(source, rejected)
        differences.append(
            [left - right for left, right in zip(chosen_features, rejected_features)]
        )
    weights = [0.0] * FEATURE_DIM
    for _ in range(epochs):
        for difference in differences:
            margin = dot(weights, difference)
            probability = 1.0 / (1.0 + math.exp(-max(-60.0, min(60.0, margin))))
            gradient = 1.0 - probability
            for index, value in enumerate(difference):
                weights[index] += learning_rate * (gradient * value - l2 * weights[index])
    return {
        "schemaVersion": 1,
        "algorithm": "geulrank-context-linear-v1",
        "featureDim": FEATURE_DIM,
        "weights": weights,
        "bias": 0.0,
    }


def quantize_model(model: dict, feature_scale: float | None = None) -> dict:
    if model.get("featureDim") != FEATURE_DIM:
        raise ValueError("model featureDim does not match the runtime contract")
    weights = [float(value) for value in model.get("weights", [])]
    if len(weights) != FEATURE_DIM:
        raise ValueError("model weights have the wrong dimension")
    maximum = max(abs(value) for value in weights)
    weight_scale = maximum / 127.0 if maximum else 1.0
    if feature_scale is None:
        feature_scale = 1.0 / 127.0
    if not math.isfinite(feature_scale) or feature_scale <= 0:
        raise ValueError("feature_scale must be positive and finite")
    quantized = [max(-127, min(127, round(value / weight_scale))) for value in weights]
    return {
        "schemaVersion": 1,
        "format": "geulrank-context-linear-int8-v1",
        "featureDim": FEATURE_DIM,
        "featureScale": feature_scale,
        "weightScale": weight_scale,
        "weights": quantized,
        "bias": float(model.get("bias", 0.0)),
    }


def quantized_score(model: dict, values: list[float]) -> float:
    if len(values) != model.get("featureDim"):
        raise ValueError("feature vector dimension does not match model")
    quantized = [max(-127, min(127, round(value / model["featureScale"]))) for value in values]
    dot_product = sum(int(left) * int(right) for left, right in zip(quantized, model["weights"]))
    return dot_product * model["featureScale"] * model["weightScale"] + model.get("bias", 0.0)


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
