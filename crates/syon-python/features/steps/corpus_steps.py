from pathlib import Path

from behave import given, then

REPO_ROOT = Path(__file__).resolve().parents[4]


@given('the SYON file "{path}"')
def step_given_syon_file(context, path):
    context.document = (REPO_ROOT / path).read_text(encoding="utf-8")


# "When it is parsed" is defined once, in spacing_rule_steps.py, and shared
# across all .feature files in this features/ directory.


@then('the value at "{key_path}" is "{expected}"')
def step_then_value_at_path(context, key_path, expected):
    node = context.result
    for key in key_path.split("."):
        assert isinstance(node, dict) and key in node, (
            f"{key_path!r}: no key {key!r} in {node!r}"
        )
        node = node[key]
    assert node == expected, f"{key_path!r} = {node!r}, want {expected!r}"
