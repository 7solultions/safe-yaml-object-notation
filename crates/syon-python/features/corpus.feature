Feature: Real corpus files parse to their documented values
  The project's own examples/ and docs/decisions/ files aren't just
  fixtures that must parse without error (see tests/test_parse.py and the
  Rust/Go examples-valid checks) -- they carry known, human-authored
  values. These scenarios pin specific fields of real files to the values
  they're documented to have.

  Scenario Outline: A real SYON file parses to its known field values
    Given the SYON file "<path>"
    When it is parsed
    Then the value at "<key path>" is "<value>"

    Examples: ADRs share a common architecture-decision-record schema
      | path                                                     | key path                                 | value                                                                      |
      | docs/decisions/0001-record-architecture-decisions.syon   | architecture-decision-record.identifier  | 0001                                                                       |
      | docs/decisions/0001-record-architecture-decisions.syon   | architecture-decision-record.title       | Record architecture decisions                                             |
      | docs/decisions/0001-record-architecture-decisions.syon   | architecture-decision-record.status      | accepted                                                                   |
      | docs/decisions/0002-pest-as-the-rust-parsing-engine.syon | architecture-decision-record.identifier  | 0002                                                                       |
      | docs/decisions/0002-pest-as-the-rust-parsing-engine.syon | architecture-decision-record.title       | Use pest as the Rust parsing engine                                       |
      | docs/decisions/0003-preflight-scan-for-forbidden-constructs.syon | architecture-decision-record.identifier | 0003                                                                |
      | docs/decisions/0004-independent-go-implementation.syon   | architecture-decision-record.identifier  | 0004                                                                       |
      | docs/decisions/0004-independent-go-implementation.syon   | architecture-decision-record.title       | Independent Go implementation instead of FFI bindings                     |
      | docs/decisions/0005-block-1-only-yaml-compatibility.syon | architecture-decision-record.identifier  | 0005                                                                       |
      | docs/decisions/0005-block-1-only-yaml-compatibility.syon | architecture-decision-record.title       | Only Block 1 is YAML-compatible                                           |
      | docs/decisions/0006-phase1-block-numbering.syon          | architecture-decision-record.identifier  | 0006                                                                       |
      | docs/decisions/0006-phase1-block-numbering.syon          | architecture-decision-record.title       | Phase1 report uses its own block numbering, distinct from the grammar spec |

    Examples: The glossary worked example (examples/glossary/entries/syon.syon)
      | path                                 | key path      | value                      |
      | examples/glossary/entries/syon.syon  | abbreviation  | SYON                       |
      | examples/glossary/entries/syon.syon  | term          | Safe YAML Object Notation  |
      | examples/glossary/entries/syon.syon  | id            | syon-001                   |
      | examples/glossary/entries/syon.syon  | version       | 0.9.0                      |

    Examples: The glossary schema (examples/glossary/schema.syon)
      | path                     | key path        | value          |
      | examples/glossary/schema.syon | schema.name | glossary-entry |
      | examples/glossary/schema.syon | schema.version | 0.1.0       |
