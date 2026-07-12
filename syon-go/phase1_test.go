package syon

import "testing"

func phase1Counts(t *testing.T, src string) *Phase1Counts {
	t.Helper()
	n, err := Parse([]byte(src))
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	c := &Phase1Counts{}
	c.AddNode(n)
	return c
}

func TestPhase1Block1OnlyIsFullyYAMLCompatible(t *testing.T) {
	c := phase1Counts(t, "name: Alice\nage: 30\ntags:\n  - a\n  - b\n")
	if c.MappingEntries != 3 { // name, age, tags
		t.Errorf("MappingEntries = %d, want 3", c.MappingEntries)
	}
	if c.SequenceItems != 2 {
		t.Errorf("SequenceItems = %d, want 2", c.SequenceItems)
	}
	if !c.YAMLCompatible() {
		t.Error("expected YAML-compatible")
	}
	if got := c.YAMLCompatibilityPercent(); got != 100 {
		t.Errorf("YAMLCompatibilityPercent = %d, want 100", got)
	}
}

func TestPhase1LiteralBlockReducesCompatibility(t *testing.T) {
	// One mapping entry (Block 1) whose value is a literal block (block2) --
	// a mix, hence 50% rather than 0%, matching the Rust analyzer.
	c := phase1Counts(t, "description: [[[\n  hello\n]]]\n")
	if c.MappingEntries != 1 {
		t.Errorf("MappingEntries = %d, want 1", c.MappingEntries)
	}
	if c.LiteralBlocks != 1 {
		t.Errorf("LiteralBlocks = %d, want 1", c.LiteralBlocks)
	}
	if c.YAMLCompatible() {
		t.Error("expected NOT YAML-compatible")
	}
	if got := c.YAMLCompatibilityPercent(); got != 50 {
		t.Errorf("YAMLCompatibilityPercent = %d, want 50", got)
	}
}

func TestPhase1InlineSymbolsAreTalliedNotDoubleCounted(t *testing.T) {
	c := phase1Counts(t, "url: https://example.com\ntag: -draft\n")
	if c.MappingEntries != 2 {
		t.Errorf("MappingEntries = %d, want 2", c.MappingEntries)
	}
	if c.SequenceItems != 0 {
		t.Errorf("SequenceItems = %d, want 0", c.SequenceItems)
	}
	if c.InlineColon < 1 {
		t.Error("expected at least one inline colon")
	}
	if c.InlineDash < 1 {
		t.Error("expected at least one inline dash")
	}
}

func TestPhase1FenceIsTalliedWithPathAndFormat(t *testing.T) {
	// A fence is the ONLY top-level construct Go's Parse can return
	// alongside it -- unlike Rust, Go has no multi-document support, so this
	// document is nothing but the fence itself.
	c := phase1Counts(t, "```config/settings.json\nkey: value\n```\n")
	if c.Fences != 1 {
		t.Errorf("Fences = %d, want 1", c.Fences)
	}
	if len(c.FencePaths) != 1 || c.FencePaths[0] != "config/settings" {
		t.Errorf("FencePaths = %v, want [config/settings]", c.FencePaths)
	}
	if len(c.FenceFormats) != 1 || c.FenceFormats[0] != "json" {
		t.Errorf("FenceFormats = %v, want [json]", c.FenceFormats)
	}
	if c.YAMLCompatible() {
		t.Error("expected NOT YAML-compatible")
	}
}

func TestPhase1EmptyDocumentIsFullyCompatibleByConvention(t *testing.T) {
	c := &Phase1Counts{}
	if got := c.YAMLCompatibilityPercent(); got != 100 {
		t.Errorf("YAMLCompatibilityPercent = %d, want 100", got)
	}
	if !c.YAMLCompatible() {
		t.Error("expected YAML-compatible")
	}
}
