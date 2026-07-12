package syon

// Phase1Counts mirrors crates/syon-parser/src/phase1.rs's analysis, walking
// an already-parsed Node tree to report Block 1/2/3 usage, a complexity
// score, and a YAML 1.2 compatibility estimate.
//
// Terminology note: LiteralBlocks is this tool's "block2" and Fences is its
// "block3" -- the OPPOSITE of spec/02-grammar.md's Block 2 (fence) / Block 3
// (literal) numbering. See docs/decisions/0006-phase1-block-numbering.syon.
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
	LiteralBlocks   uint64 // this tool's "block2"
	Fences          uint64 // this tool's "block3"
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

// YAMLCompatible reports whether this content used only Block 1 constructs.
func (c *Phase1Counts) YAMLCompatible() bool {
	return c.LiteralBlocks == 0 && c.Fences == 0
}

// YAMLCompatibilityPercent is the share of YAML-compatible (Block 1)
// constructs among all Block 1/2/3 constructs, as a rounded percentage. An
// empty document is considered 100% compatible.
func (c *Phase1Counts) YAMLCompatibilityPercent() uint64 {
	block1 := c.MappingEntries + c.SequenceItems
	total := block1 + c.LiteralBlocks + c.Fences
	if total == 0 {
		return 100
	}
	return (100*block1 + total/2) / total
}
