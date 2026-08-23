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


def test_flow_mapping_passes_through_as_scalar():
    # SYON declines to interpret `{`/`[`: the text reaches the consumer
    # verbatim, which is what keeps `{{ .TASK }}` templates intact.
    assert syon.parse("obj: {a: 1}\n")["obj"] == "{a: 1}"


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


# --- Comprehensive SyonError assertion tests ---

def test_error_yaml_tag():
    with pytest.raises(syon.SyonError, match=r"(?i)tag|forbidden|!!"):
        syon.parse("value: !!str hello\n")


def test_error_anchor():
    with pytest.raises(syon.SyonError, match=r"(?i)anchor|forbidden|&"):
        syon.parse("base: &anchor value\n")


def test_error_alias():
    with pytest.raises(syon.SyonError, match=r"(?i)alias|forbidden|\*"):
        syon.parse("ref: *anchor\n")


def test_flow_mapping_is_not_an_error():
    assert syon.parse("obj: {a: 1, b: 2}\n")["obj"] == "{a: 1, b: 2}"


def test_flow_sequence_is_not_an_error():
    assert syon.parse("list: [1, 2, 3]\n")["list"] == "[1, 2, 3]"


def test_leading_doc_marker_is_allowed_but_a_second_is_not():
    # One `---` opens the single document a SYON file holds; a second one
    # would start a stream, which SYON forbids.
    assert syon.parse("---\nkey: value\n")["key"] == "value"
    with pytest.raises(syon.SyonError, match=r"(?i)forbidden|---|second"):
        syon.parse("---\nkey: value\n---\nother: x\n")


def test_error_complex_key():
    with pytest.raises(syon.SyonError, match=r"(?i)forbidden|complex|key|\?"):
        syon.parse("? complex key\n: value\n")


def test_error_key_starting_with_operator():
    with pytest.raises(syon.SyonError):
        syon.parse(": bad key\n")


def test_error_tab_indentation():
    with pytest.raises(syon.SyonError, match=r"(?i)tab|indent"):
        syon.parse("key:\n\tchild: value\n")


def test_syon_error_is_exception():
    assert issubclass(syon.SyonError, Exception)


def test_error_has_position_info():
    try:
        syon.parse("valid: line\ninvalid: !!str boom\n")
    except syon.SyonError as e:
        msg = str(e)
        assert any(c.isdigit() for c in msg)


# --- Error codes ---


@pytest.mark.parametrize(
    "src,code",
    [
        ("key:\n\tchild: value\n", syon.ErrorCode.TAB_IN_INDENTATION),
        ("a: 1\n---\nb: 2\n", syon.ErrorCode.DOCUMENT_START_MARKER),
        ("a: 1\n...\n", syon.ErrorCode.DOCUMENT_END_MARKER),
        ("? a\n", syon.ErrorCode.COMPLEX_KEY),
        ("desc: [[[\n  x\n  ]]]\n", syon.ErrorCode.LITERAL_BLOCK_REMOVED),
        ("```path.json\nkey: value\n", syon.ErrorCode.UNTERMINATED_FENCE),
        ("key: &anchor\n", syon.ErrorCode.ANCHOR),
        ("key: *alias\n", syon.ErrorCode.ALIAS),
        ("key: !!str x\n", syon.ErrorCode.EXPLICIT_TAG),
        ("a: 1\na: 2\n", syon.ErrorCode.DUPLICATE_KEY),
    ],
)
def test_error_code_is_stable(src, code):
    """The code is API; the message wording is not.

    Mirrors `error_codes_are_stable` in crates/syon-parser/src/parser.rs and
    `TestForbiddenAndSyntax` in syon-go/syon_test.go.
    """
    with pytest.raises(syon.SyonError) as exc:
        syon.parse(src)
    assert exc.value.code == code


def test_error_carries_code_kind_and_message():
    with pytest.raises(syon.SyonError) as exc:
        syon.parse("```path.json\nkey: value\n")
    err = exc.value
    assert err.code == syon.ErrorCode.UNTERMINATED_FENCE
    assert int(err.code) == 202
    assert str(err.code) == "SYON-202"
    assert err.kind == "syntax"
    assert "unterminated ``` document fence" in err.message
    # `.message` is the bare text; `str(e)` adds the code and kind.
    assert str(err).startswith("[SYON-202] syntax error:")


def test_forbidden_and_syntax_kinds_are_distinguished():
    with pytest.raises(syon.SyonError) as forbidden:
        syon.parse("key: &anchor\n")
    assert forbidden.value.kind == "forbidden"

    with pytest.raises(syon.SyonError) as syntax:
        syon.parse("a: 1\na: 2\n")
    assert syntax.value.kind == "syntax"
