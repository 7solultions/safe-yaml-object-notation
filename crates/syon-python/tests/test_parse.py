"""Tests for the syon Python extension (crates/syon-python).

Run via `maturin develop -m crates/syon-python/Cargo.toml` (or
`task build-python-bindings`) followed by `pytest`, from within a
virtualenv where the extension is installed.
"""
from pathlib import Path

import pytest
import syon

REPO_ROOT = Path(__file__).resolve().parents[3]


def test_parse_scalar_mapping():
    assert syon.parse("name: Alice\nage: 30\n") == {"name": "Alice", "age": "30"}


def test_strings_only_boundary():
    # No implicit type coercion: numbers/booleans/null stay strings.
    result = syon.parse("n: 42\nflag: true\nnothing: null\n")
    assert result == {"n": "42", "flag": "true", "nothing": "null"}
    assert isinstance(result["n"], str)


def test_sequence():
    assert syon.parse("items:\n  - a\n  - b\n") == {"items": ["a", "b"]}


def test_nested_mapping():
    assert syon.parse("outer:\n  inner: value\n") == {"outer": {"inner": "value"}}


def test_literal_block():
    result = syon.parse("description: [[[\n  line one\n  line two\n]]]\n")
    assert "line one" in result["description"]
    assert "line two" in result["description"]


def test_only_first_colon_space_on_a_line_is_structural():
    # See spec/01-lexer.md: only the first `: ` on a line is structural.
    result = syon.parse("key: value: with a colon: and another\n")
    assert result == {"key": "value: with a colon: and another"}


def test_dash_is_structural_only_as_first_non_space_char_of_the_line():
    # See spec/01-lexer.md: a `-` elsewhere in the line is literal text.
    assert syon.parse("note: this - is not a list item\n") == {
        "note": "this - is not a list item"
    }
    assert syon.parse("a-b: value\n") == {"a-b": "value"}


def test_forbidden_anchor_raises_value_error():
    with pytest.raises(ValueError, match="anchor"):
        syon.parse("a: &anc val\nb: *anc\n")


def test_forbidden_flow_sequence_raises_value_error():
    with pytest.raises(ValueError, match="flow"):
        syon.parse("a: [1, 2]\n")


def test_duplicate_keys_raise_value_error():
    with pytest.raises(ValueError, match="duplicate"):
        syon.parse("a: 1\na: 2\n")


def test_syon_error_carries_code_and_message():
    with pytest.raises(syon.SyonError) as exc_info:
        syon.parse("a: &anc val\nb: *anc\n")
    err = exc_info.value
    assert isinstance(err, ValueError)
    assert err.code == syon.ErrorCode.ANCHOR
    assert int(err.code) == 112
    assert "anchor" in err.message
    assert str(err) == "[SYON-112] " + err.message


def test_syon_error_codes_for_forbidden_constructs():
    cases = {
        "a: [1, 2]\n": syon.ErrorCode.FLOW_SEQUENCE,
        "a: {b: c}\n": syon.ErrorCode.FLOW_MAPPING,
        "a: !tag val\n": syon.ErrorCode.EXPLICIT_TAG,
        "a: *anc\n": syon.ErrorCode.ALIAS,
        "a: 1\na: 2\n": syon.ErrorCode.DUPLICATE_KEY,
        "---\n": syon.ErrorCode.DOCUMENT_START_MARKER,
    }
    for src, expected_code in cases.items():
        with pytest.raises(syon.SyonError) as exc_info:
            syon.parse(src)
        assert exc_info.value.code == expected_code, src


def _corpus_files():
    files = list((REPO_ROOT / "examples").rglob("*.syon"))
    files += list((REPO_ROOT / "docs" / "decisions").glob("*.syon"))
    return sorted(files)


@pytest.mark.parametrize(
    "path", _corpus_files(), ids=lambda p: str(p.relative_to(REPO_ROOT))
)
def test_corpus_file_parses(path):
    data = path.read_text(encoding="utf-8")
    syon.parse(data)  # must not raise
