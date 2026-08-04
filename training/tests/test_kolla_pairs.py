import unittest

from geullint_training.kolla_pairs import build_pair_rows, detokenize_m2


class KollaPairTests(unittest.TestCase):
    def test_converts_m2_annotation_to_a_pair_without_using_release_holdout(self):
        m2 = """S 몇일 뒤에 만나요 .
A 0 1|||R:SPELL|||며칠|||REQUIRED|||-NONE-|||0
A 0 1|||R:SPELL|||며칠|||REQUIRED|||-NONE-|||1
"""
        rows = build_pair_rows(m2, train_ratio=1.0, dev_ratio=0.0)

        self.assertEqual(len(rows), 2)
        self.assertTrue(all(row["split"] == "train" for row in rows))
        self.assertTrue(all(row["sourceText"] == "몇일 뒤에 만나요." for row in rows))
        self.assertTrue(all(row["chosenText"] == "며칠 뒤에 만나요." for row in rows))
        self.assertTrue(all(row["rejectedText"] == "몇일 뒤에 만나요." for row in rows))
        self.assertTrue(all(row["origin"] == "kolla_annotation" for row in rows))

    def test_detokenizer_preserves_punctuation_boundaries(self):
        self.assertEqual(detokenize_m2("오늘 은 비가 옵니다 ."), "오늘 은 비가 옵니다.")


if __name__ == "__main__":
    unittest.main()
