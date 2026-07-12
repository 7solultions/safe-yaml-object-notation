import ast

from behave import given, then, when

import syon


@given("the SYON document")
def step_given_document(context):
    context.document = context.text


@when("it is parsed")
def step_when_parsed(context):
    context.result = syon.parse(context.document)


@then('the result is a mapping with key "{key}" and value "{value}"')
def step_then_mapping_value(context, key, value):
    actual = context.result[key]
    assert actual == value, f"{actual!r} != {value!r}"


@then("the result is the sequence {expected}")
def step_then_sequence(context, expected):
    actual = context.result
    want = ast.literal_eval(expected)
    assert actual == want, f"{actual!r} != {want!r}"
