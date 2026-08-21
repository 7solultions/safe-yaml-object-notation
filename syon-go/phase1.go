package syon

// Phase1Counts mirrors crates/syon-parser/src/phase1.rs's analysis, walking
// an already-parsed Node tree to report block usage, a complexity score, and
// a YAML 1.2 compatibility estimate.
//
// Terminology note: this tool once numbered its own blocks the opposite way
// round from spec/02-grammar.md. That divergence is gone -- `[[[ … ]]]` was
// removed from the language, leaving Block 1 (records, including `|` block
// scalars) and Block 2 (the document fence), numbered the same in both. See
// docs/decisions/0006-phase1-block-numbering.syon.
//
// Known cross-implementation gaps versus the Rust analyzer:
//   - This package's Node does not retain comments in the AST ("future
//     work" per README.md), so Comments is always 0 here.
//   - Parse only ever returns a single top-level construct (no multi-fence
//     documents like Rust's SyonFile.documents), so at most one Fence is
//     ever observed per parse.
type Phase1Counts struct {
	MappingEntries  uint64
	SequenceItems   uint64
	Comments        uint64 // always 0: this AST does not retain comments
	LiteralBlocks   uint64 // `|` block scalars: valid YAML, so compatible
	Fences          uint64 // ```path.format document fences (Block 2)
	MaxNestingDepth uint64
	InlineColon     uint64
	InlineDash      uint64
	InlineHash      uint64
	FenceFormats    []string
	FencePaths      []string
}

// AddNode analyzes n (and everything beneath it), accumulating into c.
func (c *Phase1Counts) AddNode(n *Node) {
	c.addNode(n, 0)
}

func (c *Phase1Counts) addNode(n *Node, depth uint64) {
	if n == nil {
		return
	}
	if depth > c.MaxNestingDepth {
		c.MaxNestingDepth = depth
	}
	switch n.Kind {
	case ScalarNode:
		c.tally(n.Str)
	case LiteralNode:
		c.LiteralBlocks++
		c.tally(n.Str)
	case FenceNode:
		c.Fences++
		if n.Format != "" {
			c.FenceFormats = append(c.FenceFormats, n.Format)
		}
		if n.Path != "" {
			c.FencePaths = append(c.FencePaths, n.Path)
		}
		c.tally(n.Str)
	case MappingNode:
		for _, k := range n.Keys {
			c.MappingEntries++
			c.tally(k)
			c.addNode(n.Map[k], depth+1)
		}
	case SequenceNode:
		for _, item := range n.Seq {
			c.SequenceItems++
			c.addNode(item, depth+1)
		}
	}
}

func (c *Phase1Counts) tally(s string) {
	for _, ch := range s {
		switch ch {
		case ':':
			c.InlineColon++
		case '-':
			c.InlineDash++
		case '#':
			c.InlineHash++
		}
	}
}

// Complexity is a simple, documented score -- see the Rust analyzer for the
// same formula and rationale.
func (c *Phase1Counts) Complexity() uint64 {
	return c.MappingEntries + c.SequenceItems + c.Comments +
		c.LiteralBlocks*3 + c.Fences*4 + c.MaxNestingDepth*2
}

// YAMLCompatible reports whether this content is a strict YAML 1.2 subset.
//
// The document fence is the only remaining construct a YAML 1.2 parser cannot
// read. `|` block scalars are ordinary YAML, and `[[[ … ]]]`, which was not,
// no longer exists.
func (c *Phase1Counts) YAMLCompatible() bool {
	return c.Fences == 0
}

// YAMLCompatibilityPercent is the share of YAML-compatible constructs among
// all constructs, as a rounded percentage. An empty document is considered
// 100% compatible. Block scalars count as compatible.
func (c *Phase1Counts) YAMLCompatibilityPercent() uint64 {
	compatible := c.MappingEntries + c.SequenceItems + c.LiteralBlocks
	total := compatible + c.Fences
	if total == 0 {
		return 100
	}
	return (100*compatible + total/2) / total
}
