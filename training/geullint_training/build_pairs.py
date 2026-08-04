#!/usr/bin/env python3
"""Build document-disjoint pairwise ranking examples from reviewed candidates."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Iterable

ALLOWED_SPLITS = {"train", "dev", "release_holdout"}


def split_by_document(records: Iterable[dict]) -> dict[str, str]:
    groups: dict[str, str] = {}
    for record in records:
        document_id = record.get("documentId")
        split = record.get("split")
        if not isinstance(document_id, str) or not document_id.strip():
            raise ValueError("each record requires a non-empty documentId")
        if split not in ALLOWED_SPLITS:
            raise ValueError(f"unsupported split: {split}")
        previous = groups.get(document_id)
        if previous is not None and previous != split:
            raise ValueError(f"document {document_id} appears in multiple splits")
        groups[document_id] = split
    return groups


def build_pair_rows(records: Iterable[dict]) -> list[dict]:
    records = list(records)
    split_by_document(records)
    pairs: list[dict] = []
    for record in records:
        if record.get("split") == "release_holdout":
            continue
        candidates = record.get("candidates")
        if not isinstance(candidates, list) or not candidates:
            raise ValueError(f"record {record.get('id')} requires candidates")
        positives = [candidate for candidate in candidates if candidate.get("label") == 1]
        negatives = [candidate for candidate in candidates if candidate.get("label") == 0]
        if not positives or not negatives:
            raise ValueError(f"record {record.get('id')} requires both positive and negative candidates")
        for positive in positives:
            for negative in negatives:
                if not isinstance(positive.get("text"), str) or not isinstance(negative.get("text"), str):
                    raise ValueError(f"record {record.get('id')} candidates require text")
                pairs.append(
                    {
                        "id": f"{record['id']}::{positive['text']}::{negative['text']}",
                        "caseId": record["id"],
                        "documentId": record["documentId"],
                        "split": record["split"],
                        "chosen": positive,
                        "rejected": negative,
                    }
                )
    return pairs


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number} is not valid JSON: {error}") from error
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    pairs = build_pair_rows(read_jsonl(args.input))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("".join(json.dumps(pair, ensure_ascii=False, sort_keys=True) + "\n" for pair in pairs), encoding="utf-8")
    print(json.dumps({"pairs": len(pairs), "output": str(args.output)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
