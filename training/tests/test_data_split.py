import unittest

from geullint_training.build_pairs import build_pair_rows, split_by_document


class DataSplitTests(unittest.TestCase):
    def test_keeps_documents_whole_and_never_leaks_release_holdout(self):
        records = [
            {"id": "a", "documentId": "doc-a", "split": "train", "candidates": [{"text": "며칠", "label": 1}, {"text": "몇일", "label": 0}]},
            {"id": "b", "documentId": "doc-a", "split": "train", "candidates": [{"text": "읽는다", "label": 1}, {"text": "읽은다", "label": 0}]},
            {"id": "c", "documentId": "doc-holdout", "split": "release_holdout", "candidates": [{"text": "문장", "label": 1}, {"text": "문쟝", "label": 0}]},
        ]
        groups = split_by_document(records)
        self.assertEqual(groups["doc-a"], "train")
        self.assertEqual(groups["doc-holdout"], "release_holdout")
        pairs = build_pair_rows(records)
        self.assertEqual(len(pairs), 2)
        self.assertTrue(all(pair["documentId"] != "doc-holdout" for pair in pairs))

    def test_rejects_duplicate_document_split_and_missing_labels(self):
        with self.assertRaises(ValueError):
            split_by_document([
                {"id": "a", "documentId": "doc", "split": "train"},
                {"id": "b", "documentId": "doc", "split": "dev"},
            ])
        with self.assertRaises(ValueError):
            build_pair_rows([{"id": "a", "documentId": "doc", "split": "train", "candidates": [{"text": "x"}]}])


if __name__ == "__main__":
    unittest.main()
