import pytest
import syon


def test_simple_mapping():
    result = syon.parse("name: beriah\nversion: 1.0\n")
    assert result["name"] == "beriah"
    assert result["version"] == "1.0"


def test_colon_no_space_is_literal():
    result = syon.parse("url: http://example.com\n")
    assert result["url"] == "http://example.com"


def test_sequence():
    result = syon.parse("tags:\n  - rust\n  - python\n")
    assert result["tags"] == ["rust", "python"]


def test_nested_mapping():
    src = "settings:\n  strict: true\n  retries: 3\n"
    result = syon.parse(src)
    assert result["settings"]["strict"] == "true"
    assert result["settings"]["retries"] == "3"


def test_comment_stripped():
    result = syon.parse("name: beriah  # trailing\n")
    assert result["name"] == "beriah"


def test_block_scalar():
    src = "notes: |\n  line one\n  line two\n"
    result = syon.parse(src)
    assert "line one" in result["notes"]
    assert "line two" in result["notes"]


def test_rejects_yaml_tag():
    with pytest.raises(syon.SyonError):
        syon.parse("value: !!str hello\n")


def test_rejects_anchor():
    with pytest.raises(syon.SyonError):
        syon.parse("base: &anchor value\n")


def test_rejects_flow_mapping():
    with pytest.raises(syon.SyonError):
        syon.parse("obj: {a: 1}\n")


def test_multi_document():
    src = "```first.syon\nkey: one\n```second.syon\nkey: two\n"
    docs = syon.parse_documents(src)
    assert len(docs) == 2
    assert docs[0]["key"] == "one"
    assert docs[1]["key"] == "two"
