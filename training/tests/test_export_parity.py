import unittest

from geullint_training.export_onnx import dequantize_weights, quantize_weights


class ExportParityTests(unittest.TestCase):
    def test_int8_quantization_round_trips_with_bounded_error(self):
        weights = {"bias": 0.5, "edit_distance": -0.9, "frequency": 0.18}
        quantized, scale = quantize_weights(weights)
        restored = dequantize_weights(quantized, scale)
        self.assertEqual(set(quantized), set(weights))
        self.assertLess(max(abs(restored[key] - value) for key, value in weights.items()), 0.02)


if __name__ == "__main__":
    unittest.main()
