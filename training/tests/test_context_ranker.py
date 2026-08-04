import unittest
from pathlib import Path
import importlib.util

from geullint_training.context_ranker import (
    FEATURE_DIM,
    feature_vector,
    quantize_feature_vector,
    quantize_model,
    quantized_score,
    train_pairwise,
)


class ContextRankerFeatureTests(unittest.TestCase):
    def test_features_are_deterministic_and_include_context(self):
        source = "몇일 뒤에 만나요."
        chosen = feature_vector(source, "며칠 뒤에 만나요.")
        repeated = feature_vector(source, "며칠 뒤에 만나요.")
        different = feature_vector(source, "몇 일 뒤에 만나요.")

        self.assertEqual(len(chosen), FEATURE_DIM)
        self.assertEqual(chosen, repeated)
        self.assertNotEqual(chosen, different)
        self.assertTrue(any(value != 0.0 for value in chosen))

    def test_feature_quantization_is_bounded_and_repeatable(self):
        values = feature_vector("저는 문장을 검사합니다.", "저는 문장을 점검합니다.")
        quantized, scale = quantize_feature_vector(values)
        again, same_scale = quantize_feature_vector(values)

        self.assertEqual(quantized, again)
        self.assertEqual(scale, same_scale)
        self.assertEqual(len(quantized), FEATURE_DIM)
        self.assertTrue(all(-127 <= value <= 127 for value in quantized))
        self.assertGreater(scale, 0.0)

    def test_pairwise_training_is_deterministic_and_prefers_the_chosen_candidate(self):
        rows = [
            {
                "id": "case-1",
                "sourceText": "몇일 뒤에 만나요.",
                "chosenText": "며칠 뒤에 만나요.",
                "rejectedText": "몇일 뒤에 만나요.",
                "split": "train",
            },
            {
                "id": "case-2",
                "sourceText": "문장을 검사합니다.",
                "chosenText": "문장을 점검합니다.",
                "rejectedText": "문장을 검사함니다.",
                "split": "train",
            },
        ]
        first = quantize_model(train_pairwise(rows, epochs=4))
        second = quantize_model(train_pairwise(rows, epochs=4))
        self.assertEqual(first, second)
        chosen = quantized_score(first, feature_vector(rows[0]["sourceText"], rows[0]["chosenText"]))
        rejected = quantized_score(first, feature_vector(rows[0]["sourceText"], rows[0]["rejectedText"]))
        self.assertGreater(chosen, rejected)

    def test_pairwise_training_rejects_release_holdout_rows(self):
        with self.assertRaises(ValueError):
            train_pairwise(
                [
                    {
                        "sourceText": "원문",
                        "chosenText": "교정문",
                        "rejectedText": "원문",
                        "split": "release_holdout",
                    }
                ]
            )

    @unittest.skipUnless(importlib.util.find_spec("onnxruntime"), "onnxruntime optional dependency is not installed")
    def test_checked_in_onnx_artifact_matches_the_json_runtime(self):
        import json

        import numpy as np
        import onnxruntime as ort

        root = Path("models/geulrank-small/context-ranker")
        model = json.loads((root / "context-linear-int8.json").read_text(encoding="utf-8"))
        source = "몇일 뒤에 만나요."
        candidate = "며칠 뒤에 만나요."
        values = feature_vector(source, candidate)
        features = np.asarray(
            [np.clip(np.rint(np.asarray(values) / model["featureScale"]), -127, 127)],
            dtype=np.int8,
        )
        session = ort.InferenceSession(str(root / "context-linear-int8.onnx"), providers=["CPUExecutionProvider"])
        onnx_score = float(session.run(None, {"features": features})[0][0][0])
        self.assertAlmostEqual(onnx_score, quantized_score(model, values), places=5)


if __name__ == "__main__":
    unittest.main()
