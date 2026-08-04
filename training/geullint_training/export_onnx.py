#!/usr/bin/env python3
"""Export portable int8 weights and a runtime manifest.

The current core consumes the same named weights directly. If the optional `onnx` dependency is
installed, a future exporter can add a graph without changing this quantization contract.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def quantize_weights(weights: dict[str, float]) -> tuple[dict[str, int], float]:
    if not weights:
        raise ValueError("weights must not be empty")
    maximum = max(abs(float(value)) for value in weights.values())
    scale = maximum / 127.0 if maximum else 1.0
    return {key: max(-127, min(127, round(float(value) / scale))) for key, value in weights.items()}, scale


def dequantize_weights(weights: dict[str, int], scale: float) -> dict[str, float]:
    if scale <= 0:
        raise ValueError("scale must be positive")
    return {key: int(value) * scale for key, value in weights.items()}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    model = json.loads(args.input.read_text(encoding="utf-8"))
    quantized, scale = quantize_weights(model["weights"])
    payload = json.dumps({"schemaVersion": 1, "weights": quantized, "scale": scale}, sort_keys=True).encode("utf-8")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    artifact = args.out_dir / "geulrank-small.int8.json"
    artifact.write_bytes(payload)
    manifest = {
        "schemaVersion": 1,
        "name": "GeulRank-small",
        "format": "geulrank-linear-int8-v1",
        "features": model.get("features", []),
        "artifact": artifact.name,
        "bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "scale": scale,
        "trainingManifestSha256": hashlib.sha256(args.input.read_bytes()).hexdigest(),
        "license": "MIT",
    }
    (args.out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
