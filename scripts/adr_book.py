#!/usr/bin/env python3
"""Collect every ADR into one JSON file for the Typst book template.

The ADRs are SYON documents, so they are read through `syon-cli` rather than
by a second parser here -- the book is built from the same bytes CI validates.
Prose is structured on this side (paragraphs, indented code blocks, inline
`code` spans) so the Typst template only has to lay out, not to parse.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ADR_DIR = os.path.join(ROOT, "design", "architecture")
INDEX = os.path.join(ADR_DIR, "README.md")
OUT = os.path.join(ROOT, "build", "adr", "adrs.json")

# A run of literal backticks (```) is prose about fences, not a code span.
SPAN = re.compile(r"`{2,}|`([^`\n]+)`")


def runs(text):
    """Split a line into alternating text and inline-code runs."""
    text = text.replace(" -- ", " — ")
    out, pos = [], 0
    for m in SPAN.finditer(text):
        if m.start() > pos:
            out.append({"t": "text", "v": text[pos:m.start()]})
        if m.group(1) is None:
            out.append({"t": "text", "v": m.group(0)})
        else:
            out.append({"t": "code", "v": m.group(1)})
        pos = m.end()
    if pos < len(text):
        out.append({"t": "text", "v": text[pos:]})
    return out or [{"t": "text", "v": ""}]


def blocks(text):
    """Split a block scalar into paragraphs and indented code blocks."""
    out = []
    for para in (text or "").split("\n\n"):
        lines = [l for l in para.split("\n") if l.strip()]
        if not lines:
            continue
        if all(l.startswith("    ") for l in lines):
            out.append({"kind": "code", "text": "\n".join(l[4:] for l in lines)})
        else:
            out.append({"kind": "para", "runs": runs(" ".join(l.strip() for l in lines))})
    return out


def order():
    """Canonical order comes from the index table, so the book cannot drift."""
    ids = re.findall(r"^\| \[([a-z_0-9]+)\]\(([^)]+)\)", open(INDEX).read(), re.M)
    return [(i, os.path.join(ADR_DIR, f)) for i, f in ids]


def main():
    subprocess.run(["cargo", "build", "-q", "-p", "syon-cli"], cwd=ROOT, check=True)
    cli = os.path.join(ROOT, "target", "debug", "syon")

    on_disk = {f for f in os.listdir(ADR_DIR) if f.endswith(".syon")}
    records, listed = [], set()

    for identifier, path in order():
        listed.add(os.path.basename(path))
        raw = subprocess.run([cli, path], capture_output=True, text=True, check=True).stdout
        r = json.loads(raw)["architecture-decision-record"]
        if r["identifier"] != identifier:
            sys.exit(f"index says {identifier}, record says {r['identifier']}")
        cons = r.get("consequences") or {}
        records.append({
            "id": r["identifier"],
            "title": runs(r["title"]),
            "title_plain": r["title"],
            "status": r["status"],
            "date": r["date"],
            "deciders": r.get("deciders") or [],
            "superseded_by": r.get("superseded-by") or "",
            "amended_by": r.get("amended-by") or "",
            "open_questions": [runs(q) for q in (r.get("open-questions") or [])],
            "context": blocks(r.get("context")),
            "decision": blocks(r.get("decision")),
            "outcome": blocks(r["outcome"]) if r.get("outcome") else [],
            "positive": [runs(c) for c in (cons.get("positive") or [])],
            "negative": [runs(c) for c in (cons.get("negative") or [])],
            "rejected": [{"name": runs(a["name"]), "reason": runs(a["reason"])}
                         for a in (r.get("alternatives-rejected") or [])],
        })

    missing = on_disk - listed
    if missing:
        sys.exit("not listed in the index: " + ", ".join(sorted(missing)))

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w") as fh:
        json.dump({"records": records}, fh, indent=1)
    print(f"{len(records)} records -> {os.path.relpath(OUT, ROOT)}")


if __name__ == "__main__":
    main()
