"""Train and export the reproducible local context ranker."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from .context_ranker import (
    FEATURE_DIM,
    feature_vector,
    quantize_model,
    train_pairwise,
    write_json,
)


def read_rows(path: Path) -> list[dict]:
    rows: list[dict] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number} is not valid JSON: {error}") from error
        if not isinstance(row, dict):
            raise ValueError(f"{path}:{line_number} must be a JSON object")
        rows.append(row)
    if not rows:
        raise ValueError("training input must contain at least one row")
    return rows


def feature_scale_for_rows(rows: list[dict]) -> float:
    maximum = 0.0
    for row in rows:
        source = row.get("sourceText")
        chosen = row.get("chosenText")
        rejected = row.get("rejectedText")
        if not all(isinstance(value, str) for value in (source, chosen, rejected)):
            raise ValueError("each row requires sourceText, chosenText, and rejectedText")
        maximum = max(maximum, *(abs(value) for value in feature_vector(source, chosen)))
        maximum = max(maximum, *(abs(value) for value in feature_vector(source, rejected)))
    return maximum / 127.0 if maximum else 1.0 / 127.0


def export_int8_onnx(path: Path, model: dict) -> None:
    try:
        import onnx
        from onnx import TensorProto, helper
    except ImportError as error:  # pragma: no cover - exercised in an optional environment
        raise RuntimeError("install training[onnx] before exporting the ONNX ranker") from error

    import numpy as np

    quantized = model["weights"]
    dimension = model["featureDim"]
    graph_inputs = [
        helper.make_tensor_value_info("features", TensorProto.INT8, [None, dimension]),
    ]
    graph_outputs = [helper.make_tensor_value_info("score", TensorProto.FLOAT, [None, 1])]
    initializers = [
        helper.make_tensor("weights", TensorProto.INT8, [dimension, 1], np.asarray(quantized, dtype=np.int8).reshape(dimension, 1).tobytes(), raw=True),
        helper.make_tensor("feature_zero_point", TensorProto.INT8, [], [0]),
        helper.make_tensor("weight_zero_point", TensorProto.INT8, [], [0]),
        helper.make_tensor("output_scale", TensorProto.FLOAT, [], [model["featureScale"] * model["weightScale"]]),
        helper.make_tensor("bias", TensorProto.FLOAT, [], [model["bias"]]),
    ]
    nodes = [
        helper.make_node("MatMulInteger", ["features", "weights", "feature_zero_point", "weight_zero_point"], ["integer_score"]),
        helper.make_node("Cast", ["integer_score"], ["float_score"], to=TensorProto.FLOAT),
        helper.make_node("Mul", ["float_score", "output_scale"], ["scaled_score"]),
        helper.make_node("Add", ["scaled_score", "bias"], ["score"]),
    ]
    graph = helper.make_graph(nodes, "geulrank-context-linear-int8", graph_inputs, graph_outputs, initializers)
    model_proto = helper.make_model(graph, producer_name="geullint-training", opset_imports=[helper.make_opsetid("", 13)])
    onnx.checker.check_model(model_proto)
    path.parent.mkdir(parents=True, exist_ok=True)
    onnx.save_model(model_proto, path)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--epochs", type=int, default=24)
    parser.add_argument("--learning-rate", type=float, default=0.05)
    args = parser.parse_args()

    rows = read_rows(args.input)
    model = train_pairwise(rows, epochs=args.epochs, learning_rate=args.learning_rate)
    quantized = quantize_model(model, feature_scale_for_rows(rows))
    output = args.out_dir
    output.mkdir(parents=True, exist_ok=True)
    json_path = output / "context-linear-int8.json"
    onnx_path = output / "context-linear-int8.onnx"
    write_json(json_path, quantized)
    export_int8_onnx(onnx_path, quantized)
    manifest = {
        "schemaVersion": 1,
        "name": "GeulRank-small-context",
        "version": "0.1.0-kolla-train",
        "format": quantized["format"],
        "runtime": "geulrank-context-linear-int8-v1",
        "onnx": True,
        "featureDim": FEATURE_DIM,
        "featureScale": quantized["featureScale"],
        "weightScale": quantized["weightScale"],
        "jsonArtifact": json_path.name,
        "onnxArtifact": onnx_path.name,
        "jsonSha256": hashlib.sha256(json_path.read_bytes()).hexdigest(),
        "onnxSha256": hashlib.sha256(onnx_path.read_bytes()).hexdigest(),
        "trainingInputSha256": hashlib.sha256(args.input.read_bytes()).hexdigest(),
        "training": {
            "rows": len(rows),
            "epochs": args.epochs,
            "releaseHoldoutExcluded": True,
            "source": "KoLLA v2 annotations; training-only, not independently adjudicated gold",
        },
    }
    write_json(output / "manifest.json", manifest)
    print(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
