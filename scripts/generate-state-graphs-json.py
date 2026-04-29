#!/usr/bin/env python3
"""Generate the JSON state-graph bundle from the canonical YAML source.

The repository treats ``docs/skyjoust-state-graphs.yaml`` as the readable
source of truth and checks in a generated JSON mirror for consumers that prefer
JSON. This script reads the YAML source, serializes it as indented UTF-8 JSON,
and writes the target file.

Examples
--------
Generate the checked-in bundle from the repository root:

    python3 scripts/generate-state-graphs-json.py \
        docs/skyjoust-state-graphs.yaml \
        docs/skyjoust-state-graphs.json

Generate a temporary comparison file:

    python3 scripts/generate-state-graphs-json.py source.yaml target.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import yaml


def main() -> int:
    """Run the state-graph JSON generator from command-line arguments.

    Parameters
    ----------
    None
        The function reads ``sys.argv`` directly. The expected invocation is
        ``generate-state-graphs-json.py <source.yaml> <target.json>``.

    Returns
    -------
    int
        Exit code ``0`` when generation succeeds. Exit code ``2`` when the
        invocation does not provide exactly two positional arguments.

    Side Effects
    ------------
    Reads the YAML source file, writes the JSON target file, and logs usage
    errors to stderr.
    """
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
