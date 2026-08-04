"""Convert KoLLA M2 annotations into document-disjoint ranking pairs.

The resulting rows are training material only. They retain the source
annotation and are deliberately marked ``kolla_annotation`` instead of being
treated as independently adjudicated release gold.
"""

from __future__ import annotations

import re
import argparse
from pathlib import Path
from typing import Iterable


_PUNCTUATION = re.compile(r"\s+([,.;!?])", re.UNICODE)
_OPENING = re.compile(r"([([{\"'])\s+", re.UNICODE)
_CLOSING = re.compile(r"\s+([)\]}\"'])", re.UNICODE)


def detokenize_m2(sentence: str) -> str:
    """Detokenize the punctuation conventions used by M2 files."""

    if not isinstance(sentence, str):
        raise TypeError("sentence must be a string")
    value = " ".join(sentence.strip().split())
    value = _PUNCTUATION.sub(r"\1", value)
    value = _OPENING.sub(r"\1", value)
    value = _CLOSING.sub(r"\1", value)
    return value


def _groups(m2: str) -> list[list[str]]:
    if not isinstance(m2, str):
        raise TypeError("m2 must be a string")
    return [
        group.splitlines()
        for group in re.split(r"\n\s*\n", m2.replace("\r\n", "\n"))
        if group.strip()
    ]


def _split_for(index: int, total: int, train_ratio: float, dev_ratio: float) -> str:
    if total <= 0:
        raise ValueError("m2 must contain at least one sentence group")
    if not 0.0 < train_ratio <= 1.0:
        raise ValueError("train_ratio must be in (0, 1]")
    if not 0.0 <= dev_ratio <= 1.0 or train_ratio + dev_ratio > 1.0:
        raise ValueError("train_ratio and dev_ratio must leave a non-negative holdout")
    fraction = index / total
    if fraction < train_ratio:
        return "train"
    if fraction < train_ratio + dev_ratio:
        return "dev"
    return "release_holdout"


def _annotation_edit(line: str) -> tuple[int, int, str, str] | None:
    fields = line[2:].split("|||")
    if len(fields) < 3:
        return None
    try:
        start, end = (int(value) for value in fields[0].strip().split())
    except (TypeError, ValueError):
        return None
    if start < 0 or end < start:
        return None
    category = fields[1].strip()
    if not category or category == "noop":
        return None
    replacement = fields[2].strip()
    if replacement == "-NONE-":
        replacement = ""
    return start, end, replacement, category


def build_pair_rows(
    m2: str,
    *,
    train_ratio: float = 0.8,
    dev_ratio: float = 0.2,
) -> list[dict]:
    """Build one positive-vs-source pair for every local M2 annotation.

    A group is the unit of splitting. No row is ever assigned to training and
    release holdout at the same time, and no source row is promoted to a human
    adjudicated gold case by this function.
    """

    groups = _groups(m2)
    rows: list[dict] = []
    for group_index, lines in enumerate(groups):
        source_line = next((line for line in lines if line.startswith("S ")), None)
        if source_line is None:
            continue
        source_tokens = source_line[2:].strip().split()
        source_text = detokenize_m2(" ".join(source_tokens))
        split = _split_for(group_index, len(groups), train_ratio, dev_ratio)
        for annotation_index, line in enumerate(line for line in lines if line.startswith("A ")):
            edit = _annotation_edit(line)
            if edit is None:
                continue
            start, end, replacement, category = edit
            if start > len(source_tokens) or end > len(source_tokens):
                continue
            corrected_tokens = source_tokens[:start] + replacement.split() + source_tokens[end:]
            corrected_text = detokenize_m2(" ".join(corrected_tokens))
            if corrected_text == source_text:
                continue
            fields = line[2:].split("|||")
            annotator = fields[-1].strip() if fields else "unknown"
            rows.append(
                {
                    "id": f"kolla-{group_index + 1}-annotation-{annotation_index + 1}",
                    "documentId": f"kolla-group-{group_index + 1}",
                    "sourceText": source_text,
                    "chosenText": corrected_text,
                    "rejectedText": source_text,
                    "origin": "kolla_annotation",
                    "split": split,
                    "category": category,
                    "annotator": annotator,
                }
            )
    return rows


def pair_rows_to_jsonl(rows: Iterable[dict]) -> str:
    """Serialize rows in a stable order for a training manifest."""

    import json

    return "".join(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--train-ratio", type=float, default=0.8)
    parser.add_argument("--dev-ratio", type=float, default=0.2)
    args = parser.parse_args()
    rows = build_pair_rows(
        args.input.read_text(encoding="utf-8"),
        train_ratio=args.train_ratio,
        dev_ratio=args.dev_ratio,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(pair_rows_to_jsonl(rows), encoding="utf-8")
    print(f"wrote {len(rows)} training rows to {args.output}")


if __name__ == "__main__":
    main()
