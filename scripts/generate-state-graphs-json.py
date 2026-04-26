#!/usr/bin/env python3
"""Generate the JSON state-graph bundle from the canonical YAML source."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import yaml


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: generate-state-graphs-json.py <source.yaml> <target.json>",
            file=sys.stderr,
        )
        return 2

    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    data = yaml.safe_load(source.read_text(encoding="utf-8"))
    target.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
