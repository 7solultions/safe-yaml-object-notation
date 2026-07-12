Feature: The spacing rule
  As described in spec/01-lexer.md, a colon, dash, or hash is structural
  only in a specific position and only when followed by whitespace or EOL.
  These scenarios mirror that spec's examples directly, as executable
  documentation.

  Scenario: A colon followed by a space separates a mapping key from its value
    Given the SYON document
      """
      key: value
      """
    When it is parsed
    Then the result is a mapping with key "key" and value "value"

  Scenario: A colon not followed by a space is literal value text
    Given the SYON document
      """
      url: https://example.com
      """
    When it is parsed
    Then the result is a mapping with key "url" and value "https://example.com"

  Scenario: Only the first colon-space on a line is structural
    Given the SYON document
      """
      key: value: with a colon: and another
      """
    When it is parsed
    Then the result is a mapping with key "key" and value "value: with a colon: and another"

  Scenario: A dash followed by a space at the start of a line is a list item
    Given the SYON document
      """
      - alpha
      - beta
      """
    When it is parsed
    Then the result is the sequence ["alpha", "beta"]

  Scenario: A dash not followed by a space is literal value text
    Given the SYON document
      """
      tag: -draft
      """
    When it is parsed
    Then the result is a mapping with key "tag" and value "-draft"

  Scenario: A dash later in the line is literal, even when space-adjacent
    Given the SYON document
      """
      note: this - is not a list item
      """
    When it is parsed
    Then the result is a mapping with key "note" and value "this - is not a list item"

  Scenario: A dash inside a key is literal, not a list marker
    Given the SYON document
      """
      a-b: value
      """
    When it is parsed
    Then the result is a mapping with key "a-b" and value "value"

  Scenario: A hash not preceded by a space is literal value text
    Given the SYON document
      """
      id: abc#123
      """
    When it is parsed
    Then the result is a mapping with key "id" and value "abc#123"

  Scenario: A hash preceded by a space starts a trailing comment, excluded from the value
    Given the SYON document
      """
      key: value  # trailing comment
      """
    When it is parsed
    Then the result is a mapping with key "key" and value "value"
