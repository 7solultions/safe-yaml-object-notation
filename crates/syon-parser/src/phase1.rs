//! Phase 1 analysis: evaluate a SYON document's use of each block type and
//! report a complexity score plus a YAML 1.2 compatibility estimate.
//!
//! Terminology note: this module's "block2"/"block3" labels intentionally do
//! NOT match `spec/02-grammar.md`'s Block 2 (document fence) / Block 3
//! (literal escape hatch) numbering -- here, block2 is the literal block
//! (`[[[ ... ]]]`) and block3 is the document fence (` ``` `). See
//! `docs/decisions/0006-phase1-block-numbering.syon`.

use crate::ast::{Document, SyonFile, Value};

/// Per-file (or per-corpus, when aggregated) counts gathered from a parsed
/// [`SyonFile`].
#[derive(Debug, Clone, Default)]
pub struct Phase1Counts {
    /// Structural `: ` usages -- one per mapping entry.
    pub mapping_entries: u64,
    /// Structural `- ` usages -- one per sequence item.
    pub sequence_items: u64,
    /// Structural `# ` usages -- leading + trailing comments.
    pub comments: u64,
    /// Count of `[[[ ... ]]]` literal blocks (this module's "block2").
    pub literal_blocks: u64,
    /// Count of ` ```path.format ` document fences (this module's "block3").
    pub fences: u64,
    /// Deepest nesting level reached (0 = top-level scalar/empty document).
    pub max_nesting_depth: u64,
    /// Occurrences of `:` inside scalar/literal/key text that were NOT a
    /// structural key separator (i.e. everywhere else).
    pub inline_colon: u64,
    /// Occurrences of `-` inside scalar/literal/key text that were NOT a
    /// structural list marker.
    pub inline_dash: u64,
    /// Occurrences of `#` inside scalar/literal/key text that were NOT a
    /// structural comment marker.
    pub inline_hash: u64,
    /// `format` values seen on document fences, in encounter order.
    pub fence_formats: Vec<String>,
    /// `path` values seen on document fences, in encounter order.
    pub fence_paths: Vec<String>,
}

impl Phase1Counts {
    /// Analyze every document in `file`, accumulating into `self`.
    pub fn add_file(&mut self, file: &SyonFile) {
        for doc in &file.documents {
            self.add_document(doc);
        }
    }

    fn add_document(&mut self, doc: &Document) {
        if doc.format.is_some() || doc.path.is_some() {
            self.fences += 1;
            if let Some(format) = &doc.format {
                self.fence_formats.push(format.clone());
            }
            if let Some(path) = &doc.path {
                self.fence_paths.push(path.clone());
            }
        }
        self.add_value(&doc.body, 0);
    }

    fn add_value(&mut self, value: &Value, depth: u64) {
        self.max_nesting_depth = self.max_nesting_depth.max(depth);
        match value {
            Value::Scalar(s) => self.tally_inline(s),
            Value::LiteralBlock(s) => {
                self.literal_blocks += 1;
                self.tally_inline(s);
            }
            Value::Mapping(entries) => {
                for entry in entries {
                    self.mapping_entries += 1;
                    self.comments +=
                        entry.leading_comments.len() as u64 + entry.trailing_comment.is_some() as u64;
                    self.tally_inline(&entry.key);
                    self.add_value(&entry.value, depth + 1);
                }
            }
            Value::Sequence(items) => {
                for item in items {
                    self.sequence_items += 1;
                    self.comments +=
                        item.leading_comments.len() as u64 + item.trailing_comment.is_some() as u64;
                    self.add_value(&item.value, depth + 1);
                }
            }
        }
    }

    fn tally_inline(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                ':' => self.inline_colon += 1,
                '-' => self.inline_dash += 1,
                '#' => self.inline_hash += 1,
                _ => {}
            }
        }
    }

    /// A simple, documented complexity score. Weights are deliberately
    /// integers, chosen so the two non-YAML-compatible block types (literal
    /// blocks, fences) and nesting depth dominate over plain Block 1 usage.
    pub fn complexity(&self) -> u64 {
        self.mapping_entries
            + self.sequence_items
            + self.comments
            + self.literal_blocks * 3
            + self.fences * 4
            + self.max_nesting_depth * 2
    }

    /// Whether this content uses only Block 1 constructs (and is therefore a
    /// strict YAML 1.2 subset -- see `docs/decisions/0005-*`).
    pub fn yaml_compatible(&self) -> bool {
        self.literal_blocks == 0 && self.fences == 0
    }

    /// Share of YAML-compatible (Block 1) constructs among all Block
    /// 1/2/3 constructs, as an integer percentage. An empty document (no
    /// constructs at all) is considered 100% compatible.
    pub fn yaml_compatibility_percent(&self) -> u64 {
        let block1 = self.mapping_entries + self.sequence_items;
        let total = block1 + self.literal_blocks + self.fences;
        if total == 0 {
            100
        } else {
            (100 * block1 + total / 2) / total // rounded integer percentage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn counts(src: &str) -> Phase1Counts {
        let file = parse(src).unwrap();
        let mut c = Phase1Counts::default();
        c.add_file(&file);
        c
    }

    #[test]
    fn block1_only_is_fully_yaml_compatible() {
        let c = counts("name: Alice\nage: 30\ntags:\n  - a\n  - b\n");
        assert_eq!(c.mapping_entries, 3); // name, age, tags
        assert_eq!(c.sequence_items, 2);
        assert_eq!(c.literal_blocks, 0);
        assert_eq!(c.fences, 0);
        assert!(c.yaml_compatible());
        assert_eq!(c.yaml_compatibility_percent(), 100);
    }

    #[test]
    fn literal_block_reduces_compatibility() {
        // One mapping entry (Block 1, compatible) whose value is a literal
        // block (this module's "block2", not compatible) -- a mix, not a
        // pure literal-block document, hence the 50% split rather than 0%.
        let c = counts("description: [[[\n  hello\n]]]\n");
        assert_eq!(c.mapping_entries, 1);
        assert_eq!(c.literal_blocks, 1);
        assert!(!c.yaml_compatible());
        assert_eq!(c.yaml_compatibility_percent(), 50);
    }

    #[test]
    fn inline_symbols_are_tallied_not_double_counted() {
        // The colon in the URL and the dash in "-draft" are literal content,
        // not structural markers -- they must not inflate mapping/sequence
        // counts, only the inline_* tallies.
        let c = counts("url: https://example.com\ntag: -draft\n");
        assert_eq!(c.mapping_entries, 2);
        assert_eq!(c.sequence_items, 0);
        assert!(c.inline_colon >= 1);
        assert!(c.inline_dash >= 1);
    }

    #[test]
    fn fence_is_tallied_with_path_and_format() {
        // NOTE: fence bodies must currently also be valid, non-forbidden
        // SYON text (e.g. real embedded JSON with `{`/`[` is rejected by
        // preflight()) -- a known gap, tracked separately from phase1.
        let c = counts("```config/settings.json\nkey: value\n```\n");
        assert_eq!(c.fences, 1);
        assert_eq!(c.fence_paths, vec!["config/settings"]);
        assert_eq!(c.fence_formats, vec!["json"]);
        assert!(!c.yaml_compatible());
    }

    #[test]
    fn empty_document_is_fully_compatible_by_convention() {
        let c = Phase1Counts::default();
        assert_eq!(c.yaml_compatibility_percent(), 100);
        assert!(c.yaml_compatible());
    }
}
