#!/usr/bin/env python3
"""Publish the rendered ADR book alongside the synced ADR index.

`task adr-pdf` writes a dated book into `build/`, which is fine for a
release artifact and useless as a link -- a documentation page needs one URL
that does not move. This copies the newest book to `docs/decisions/ADR.pdf`
and puts a download button under the index heading.

Both steps are skipped when no book has been rendered, and the button is
written into the synced copy rather than into `design/architecture/README.md`
itself, so the link exists only on a page where the file does too: the source
index is also read on GitHub, where `build/` is ignored and there is nothing
to download. Building the site therefore never requires a Typst install.
"""

import glob
import os
import re
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOOKS = os.path.join(ROOT, "build", "ADR__*.pdf")
DOCS = os.path.join(ROOT, "docs", "decisions")
INDEX = os.path.join(DOCS, "README.md")
PDF_NAME = "ADR.pdf"

DATED = re.compile(r"ADR__(\d{4}-\d{2}-\d{2})\.pdf$")


def newest_book():
    """The most recently dated book, or None if none has been rendered.

    Sorted by the date in the name rather than by mtime: a rebuild of an
    older book should not overtake a newer one.
    """
    dated = [(m.group(1), p) for p in glob.glob(BOOKS) if (m := DATED.search(p))]
    return max(dated) if dated else (None, None)


def button(date, size, records):
    """The download block, as Markdown the site's extensions already parse."""
    megabytes = size / 1_000_000
    return (
        f'!!! abstract "Every record, in one file"\n'
        f"\n"
        f"    [Download the ADR book](./{PDF_NAME}){{ .md-button .md-button--primary "
        f'download="{PDF_NAME}" }}\n'
        f"\n"
        f"    {records} records, {megabytes:.1f}&nbsp;MB, typeset from these same\n"
        f"    `.syon` sources on {date}.\n"
    )


def main():
    date, book = newest_book()
    if book is None:
        print("no ADR book in build/ -- skipping the download link "
              "(run `task adr-pdf` to render one)")
        return 0
    if not os.path.isfile(INDEX):
        print(f"no synced index at {INDEX} -- run `task docs-sync` first", file=sys.stderr)
        return 1

    shutil.copyfile(book, os.path.join(DOCS, PDF_NAME))
    records = len(glob.glob(os.path.join(DOCS, "ADR_*.syon")))
    size = os.path.getsize(book)

    text = open(INDEX, encoding="utf-8").read()
    heading = re.search(r"^# .*\n", text, re.MULTILINE)
    if heading is None:
        print(f"{INDEX} has no level-1 heading to sit under", file=sys.stderr)
        return 1
    at = heading.end()
    text = text[:at] + "\n" + button(date, size, records) + text[at:]
    open(INDEX, "w", encoding="utf-8").write(text)

    print(f"published {PDF_NAME} ({records} records, {size / 1_000_000:.1f} MB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
