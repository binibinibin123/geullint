#!/usr/bin/env python3
"""Train the small linear ranker used as a portable baseline/calibration model."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

FEATURES = ("bias", "edit_distance", "phonology_distance", "log_frequency", "base_score")


def candidate_features(candidate: dict) -> dict[str, float]:
    values = {feature: 0.0 for feature in FEATURES}
    values["bias"] = 1.0
    values["base_score"] = float(candidate.get("score", 0.0))
    for evidence in candidate.get("evidence", []):
        code = evidence.get("code")
        value = evidence.get("value")
        try:
            number = float(value)
        except (TypeError, ValueError):
            continue
        if code == "edit-distance":
            values["edit_distance"] = number
        elif code == "phonology-distance":
            values["phonology_distance"] = number
        elif code == "frequency":
            values["log_frequency"] = math.log1p(max(number, 0.0))
    return values


def train(pairs: list[dict], *, epochs: int = 8, learning_rate: float = 0.01) -> dict[str, float]:
    weights = {feature: 0.0 for feature in FEATURES}
    weights["bias"] = 1.0
    for _ in range(epochs):
        for pair in pairs:
            if pair.get("split") == "release_holdout":
                raise ValueError("release_holdout cannot be used for training")
            chosen = candidate_features(pair["chosen"])
            rejected = candidate_features(pair["rejected"])
            difference = {feature: chosen[feature] - rejected[feature] for feature in FEATURES}
            margin = sum(weights[feature] * difference[feature] for feature in FEATURES)
            loss_gradient = max(0.0, 1.0 - margin)
            for feature in FEATURES:
                weights[feature] += learning_rate * loss_gradient * difference[feature]
    return weights


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--epochs", type=int, default=8)
    args = parser.parse_args()
    pairs = [json.loads(line) for line in args.input.read_text(encoding="utf-8").splitlines() if line.strip()]
    weights = train(pairs, epochs=args.epochs)
    result = {"schemaVersion": 1, "model": "geulrank-small", "features": list(FEATURES), "weights": weights}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"pairs": len(pairs), "output": str(args.output)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
