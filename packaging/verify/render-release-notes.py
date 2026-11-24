#!/usr/bin/env python3
"""Fill packaging/release-notes.md.in from launch-verification records (specs/004 FR-029).

Usage: render-release-notes.py <version> <records-dir> <output.md>

Records are the `launch-verification-macos-*.json` files written by launch-check.sh.
Clean-device columns stay "pending" until the maintainer runs the checklist before publishing.
"""

import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def load_records(directory: pathlib.Path) -> dict:
    records = {}
    for path in sorted(directory.glob("launch-verification-macos-*.json")):
        try:
            records[path.stem.removeprefix("launch-verification-")] = json.loads(
                path.read_text(encoding="utf-8-sig")
            )
        except (OSError, json.JSONDecodeError) as err:
            records[path.stem] = {"result": f"unreadable ({err.__class__.__name__})"}
    return records


def result_of(records: dict, key: str) -> str:
    record = records.get(key)
    return record.get("result", "missing") if record else "not run"


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__)
        return 2
    version, records_dir, output = sys.argv[1], pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
    records = load_records(records_dir)
    auto = {
        "MAC_ARM_AUTO": result_of(records, "macos-arm64"),
        "MAC_X64_AUTO": result_of(records, "macos-x64"),
    }

    def status(automated: str) -> str:
        return "supported (pending clean-device check)" if automated == "pass" else "NOT VERIFIED"

    fills = {
        "@VERSION@": version,
        "@MAC_ARM_AUTO@": auto["MAC_ARM_AUTO"],
        "@MAC_X64_AUTO@": auto["MAC_X64_AUTO"],
        "@MAC_ARM@": "pending",
        "@MAC_X64@": "pending",
        "@MAC_ARM_STATUS@": status(auto["MAC_ARM_AUTO"]),
        "@MAC_X64_STATUS@": status(auto["MAC_X64_AUTO"]),
    }
    text = (ROOT / "packaging" / "release-notes.md.in").read_text(encoding="utf-8")
    for key, value in fills.items():
        text = text.replace(key, value)
    output.write_text(text, encoding="utf-8")
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
