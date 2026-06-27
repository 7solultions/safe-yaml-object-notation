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


# --- Nested structure stress tests ---

def test_sequence_of_mappings():
    src = "people:\n  -\n    name: alice\n    age: 30\n  -\n    name: bob\n    age: 25\n"
    result = syon.parse(src)
    people = result["people"]
    assert len(people) == 2
    assert people[0]["name"] == "alice"
    assert people[0]["age"] == "30"
    assert people[1]["name"] == "bob"
    assert people[1]["age"] == "25"


def test_triple_nested_map_seq_map():
    src = "config:\n  items:\n    -\n      key: value\n      extra: data\n"
    result = syon.parse(src)
    items = result["config"]["items"]
    assert len(items) == 1
    assert items[0]["key"] == "value"
    assert items[0]["extra"] == "data"


def test_sibling_sequences_at_different_depths():
    src = "top_list:\n  - a\n  - b\nnested:\n  inner_list:\n    - c\n    - d\n"
    result = syon.parse(src)
    assert result["top_list"] == ["a", "b"]
    assert result["nested"]["inner_list"] == ["c", "d"]


def test_mixed_block_scalars_and_sequence():
    src = "root:\n  label: hello\n  items:\n    - one\n    - two\n  count: 3\n"
    result = syon.parse(src)
    inner = result["root"]
    assert inner["label"] == "hello"
    assert inner["items"] == ["one", "two"]
    assert inner["count"] == "3"


def test_dedent_to_root_after_deep_nesting():
    src = "deep:\n  level1:\n    level2:\n      leaf: value\nback_at_root: yes\n"
    result = syon.parse(src)
    assert result["back_at_root"] == "yes"
    assert result["deep"]["level1"]["level2"]["leaf"] == "value"
