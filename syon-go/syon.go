// Package syon is a native Go parser for SYON (Safe YAML Object Notation) — a
// safe, simple, structured subset/relative of YAML.
//
// SYON's guarantees, per the spec (object-notation-environment/safe-yaml-object-notation):
//
//   - Scalars are strings at the parse boundary — no implicit typing (no
//     yes/no→bool, no leading-zero octals). Applications coerce types themselves;
//     Unmarshal does that coercion against your struct fields.
//   - Forbidden YAML constructs are rejected: tags (!x/!!x), anchors/aliases
//     (&a/*a), flow style ({}/[]), the ?-complex-key marker, and --- doc starts.
//   - Verbatim multi-line text uses a `|` block scalar, exactly as in YAML.
//     `>` is accepted as a spelling of `|`; SYON has no folded style.
//   - One SYON-only block: a ```path.format fenced document (embedded content
//     returned verbatim).
//   - Duplicate keys in a mapping are an error.
//
// Typical use mirrors gopkg.in/yaml.v3:
//
//	var cfg Config
//	err := syon.Unmarshal(data, &cfg)
package syon

import (
	"fmt"
	"strings"
)

// Kind is the type of a Node.
type Kind int

const (
	ScalarNode   Kind = iota // a string value (SYON does not type scalars)
	MappingNode              // an ordered key → Node mapping
	SequenceNode             // an ordered list of Nodes
	LiteralNode              // verbatim `|` block-scalar text
	FenceNode                // an embedded ```path.format document
)

// Node is a SYON value. Only the fields relevant to Kind are populated.
type Node struct {
	Kind Kind

	Str string // ScalarNode / LiteralNode content

	Keys []string         // MappingNode: key order
	Map  map[string]*Node // MappingNode: key → value

	Seq []*Node // SequenceNode items

	Path, Format string // FenceNode: info string `path.format`
	Line         int    // 1-based line where the node began
}

func (n *Node) setKey(key string, v *Node) {
	if n.Map == nil {
		n.Map = map[string]*Node{}
	}
	n.Keys = append(n.Keys, key)
	n.Map[key] = v
}

// Error is a fatal parse error carrying a 1-based position. Kind distinguishes
// safety rejections ("forbidden") from malformed input ("syntax"), and Code is
// a stable numeric identifier for the specific problem -- match on it rather
// than on Msg, whose wording is not API. See error_code.go.
type Error struct {
	Line, Col int
	Kind      string // "forbidden" | "syntax"
	Code      ErrorCode
	Msg       string
}

func (e *Error) Error() string {
	return fmt.Sprintf("syon: [%s] %s error at line %d:%d: %s", e.Code, e.Kind, e.Line, e.Col, e.Msg)
}

func forbidden(line, col int, code ErrorCode, msg string) *Error {
	return &Error{line, col, "forbidden", code, msg}
}

func syntax(line, col int, code ErrorCode, msg string) *Error {
	return &Error{line, col, "syntax", code, msg}
}

// Parse parses SYON source into a Node tree (the top level is usually a mapping).
func Parse(data []byte) (*Node, error) {
	src := strings.ReplaceAll(string(data), "\r\n", "\n")
	p := &parser{lines: strings.Split(src, "\n")}
	node, err := p.parseBlock(0)
	if err != nil {
		return nil, err
	}
	if node == nil {
		node = &Node{Kind: MappingNode} // empty document → empty mapping
	}
	// Anything left other than trivia is a structural error.
	p.skipTrivia()
	if p.pos < len(p.lines) {
		ind, ct := p.cur()
		return nil, syntax(p.pos+1, ind+1, CodeUnexpectedContent, fmt.Sprintf("unexpected content %q", ct))
	}
	return node, nil
}

// --- parser ----------------------------------------------------------------

type parser struct {
	lines []string
	pos   int
}

// cur returns the indentation and trimmed content of the current line.
func (p *parser) cur() (int, string) { return lineParts(p.lines[p.pos]) }

func lineParts(s string) (int, string) {
	n := 0
	for n < len(s) && s[n] == ' ' {
		n++
	}
	return n, strings.TrimRight(s[n:], " \t")
}

func hasTabIndent(s string) bool {
	for i := 0; i < len(s); i++ {
		switch s[i] {
		case ' ':
		case '\t':
			return true
		default:
			return false
		}
	}
	return false
}

func isBlank(content string) bool { return content == "" }

func isComment(content string) bool {
	return content == "#" || strings.HasPrefix(content, "# ")
}

// skipTrivia advances past blank lines and full-line comments.
func (p *parser) skipTrivia() error {
	for p.pos < len(p.lines) {
		line := p.lines[p.pos]
		if hasTabIndent(line) {
			return syntax(p.pos+1, 1, CodeTabInIndentation, "tab in indentation")
		}
		_, ct := lineParts(line)
		if isBlank(ct) || isComment(ct) {
			p.pos++
			continue
		}
		return nil
	}
	return nil
}

func isSeqItem(ct string) bool { return ct == "-" || strings.HasPrefix(ct, "- ") }

// mapColon returns the index of the first structural colon (":" followed by a
// space or end-of-line), or -1.
func mapColon(ct string) int {
	for i := 0; i < len(ct); i++ {
		if ct[i] == ':' && (i+1 == len(ct) || ct[i+1] == ' ') {
			return i
		}
	}
	return -1
}

func (p *parser) parseBlock(minIndent int) (*Node, error) {
	if err := p.skipTrivia(); err != nil {
		return nil, err
	}
	if p.pos >= len(p.lines) {
		return nil, nil
	}
	ind, ct := p.cur()
	if ind < minIndent {
		return nil, nil
	}
	if ct == "---" || strings.HasPrefix(ct, "--- ") {
		return nil, forbidden(p.pos+1, ind+1, CodeDocumentStartMarker, "'---' document markers are not allowed in SYON")
	}
	switch {
	case isSeqItem(ct):
		return p.parseSequence(ind)
	case mapColon(ct) >= 0:
		return p.parseMapping(ind)
	case strings.HasPrefix(ct, "```") && ind == 0:
		return p.parseFence()
	default:
		// A bare scalar as the whole block value. A block scalar always
		// hangs off a key or a list item, so there is no header case here.
		p.pos++
		return p.scalar(ct, p.pos, ind)
	}
}

func (p *parser) parseMapping(ind int) (*Node, error) {
	m := &Node{Kind: MappingNode, Line: p.pos + 1}
	for {
		if err := p.skipTrivia(); err != nil {
			return nil, err
		}
		if p.pos >= len(p.lines) {
			break
		}
		ci, ct := p.cur()
		if ci != ind || isSeqItem(ct) {
			break
		}
		colon := mapColon(ct)
		if colon < 0 {
			break
		}
		lineNo := p.pos + 1
		key := strings.TrimRight(ct[:colon], " ")
		if err := checkKey(key, lineNo, ci+1); err != nil {
			return nil, err
		}
		if _, dup := m.Map[key]; dup {
			return nil, syntax(lineNo, ci+1, CodeDuplicateKey, fmt.Sprintf("duplicate key %q", key))
		}
		rest := strings.TrimLeft(ct[colon+1:], " ")
		p.pos++

		var (
			val *Node
			err error
		)
		switch {
		case rest == "" || isComment(rest):
			val, err = p.parseBlock(ind + 1)
			if val == nil && err == nil {
				val = &Node{Kind: ScalarNode, Line: lineNo}
			}
		case blockHeaderChomp(rest) != chompNone:
			val, err = p.parseBlockScalar(ci, blockHeaderChomp(rest))
		default:
			val, err = p.scalar(rest, lineNo, colon+1)
		}
		if err != nil {
			return nil, err
		}
		m.setKey(key, val)
	}
	return m, nil
}

func (p *parser) parseSequence(ind int) (*Node, error) {
	s := &Node{Kind: SequenceNode, Line: p.pos + 1}
	for {
		if err := p.skipTrivia(); err != nil {
			return nil, err
		}
		if p.pos >= len(p.lines) {
			break
		}
		ci, ct := p.cur()
		if ci != ind || !isSeqItem(ct) {
			break
		}
		lineNo := p.pos + 1
		rest := ""
		if ct != "-" {
			rest = strings.TrimLeft(ct[1:], " ")
		}
		p.pos++

		var (
			val *Node
			err error
		)
		switch {
		case rest == "" || isComment(rest):
			val, err = p.parseBlock(ind + 1)
			if val == nil && err == nil {
				val = &Node{Kind: ScalarNode, Line: lineNo}
			}
		case blockHeaderChomp(rest) != chompNone:
			val, err = p.parseBlockScalar(ci, blockHeaderChomp(rest))
		default:
			val, err = p.scalar(rest, lineNo, ci+3)
		}
		if err != nil {
			return nil, err
		}
		s.Seq = append(s.Seq, val)
	}
	return s, nil
}

// chomp is a block scalar's trailing-newline mode.
type chomp int

const (
	chompNone  chomp = iota // not a block header at all
	chompClip               // `|`  -- one trailing newline
	chompStrip              // `|-` -- no trailing newline
	chompKeep               // `|+` -- every trailing newline
)

// blockHeaderChomp reports whether text is a block scalar header, and which
// chomping mode it carries. `>` is accepted as a spelling of `|`: SYON has no
// folded style, so YAML written for other tools keeps its meaning rather than
// silently folding newlines into spaces.
func blockHeaderChomp(text string) chomp {
	t := strings.TrimRight(text, " \t")
	if !strings.HasPrefix(t, "|") && !strings.HasPrefix(t, ">") {
		return chompNone
	}
	switch t[1:] {
	case "":
		return chompClip
	case "-":
		return chompStrip
	case "+":
		return chompKeep
	}
	return chompNone
}

// parseBlockScalar reads the verbatim body of a `|` block scalar. The header
// was on the preceding key/item line and is already consumed. The body is
// every following line indented deeper than ownerIndent, dedented by its
// common leading indentation.
func (p *parser) parseBlockScalar(ownerIndent int, c chomp) (*Node, error) {
	start := p.pos
	var raw []string
	for p.pos < len(p.lines) {
		line := p.lines[p.pos]
		if strings.TrimSpace(line) == "" {
			raw = append(raw, line)
			p.pos++
			continue
		}
		if ci, _ := p.cur(); ci <= ownerIndent {
			break
		}
		raw = append(raw, line)
		p.pos++
	}
	// Trailing blank lines belong to whatever follows, not to the block.
	for len(raw) > 0 && strings.TrimSpace(raw[len(raw)-1]) == "" {
		raw = raw[:len(raw)-1]
		p.pos--
	}
	body := dedent(raw)
	switch c {
	case chompStrip:
		body = strings.TrimRight(body, "\n")
	case chompKeep, chompClip:
		// dedent already trims to no trailing newline; clip restores one.
		if body != "" {
			body += "\n"
		}
	}
	return &Node{Kind: LiteralNode, Str: body, Line: start}, nil
}

func (p *parser) parseFence() (*Node, error) {
	open := p.lines[p.pos]
	info := strings.TrimSuffix(open[3:], " ")
	dot := strings.LastIndex(info, ".")
	if dot < 0 {
		return nil, syntax(p.pos+1, 4, CodeFenceInfoStringMalformed, "fence info string must be `path.format`")
	}
	n := &Node{Kind: FenceNode, Path: info[:dot], Format: info[dot+1:], Line: p.pos + 1}
	start := p.pos
	p.pos++
	var raw []string
	for p.pos < len(p.lines) {
		_, ct := p.cur()
		if ct == "```" {
			p.pos++
			n.Str = strings.Join(raw, "\n")
			return n, nil
		}
		raw = append(raw, p.lines[p.pos])
		p.pos++
	}
	return nil, syntax(start+1, 1, CodeUnterminatedFence, "unterminated ``` document fence")
}

// scalar builds a ScalarNode from inline value text. A double-quoted string is
// handled first (so a '#' inside quotes is not mistaken for a comment); a plain
// scalar has its trailing comment stripped and is checked for forbidden forms.
func (p *parser) scalar(text string, line, col int) (*Node, error) {
	if strings.HasPrefix(text, `"`) {
		end := -1
		for i := 1; i < len(text); i++ {
			if text[i] == '\\' {
				i++
				continue
			}
			if text[i] == '"' {
				end = i
				break
			}
		}
		if end < 0 {
			return nil, syntax(line, col, CodeUnterminatedQuotedString, "unterminated quoted string")
		}
		// Content after the closing quote (spaces / a trailing comment) is ignored.
		return &Node{Kind: ScalarNode, Str: unquote(text[:end+1]), Line: line}, nil
	}
	text = stripTrailingComment(text)
	if err := checkForbiddenValue(text, line, col); err != nil {
		return nil, err
	}
	return &Node{Kind: ScalarNode, Str: text, Line: line}, nil
}

// --- helpers ---------------------------------------------------------------

func checkKey(key string, line, col int) error {
	if key == "" {
		return syntax(line, col, CodeEmptyKey, "empty key")
	}
	switch key[0] {
	case ':', '-', '#':
		return syntax(line, col, CodeKeyStartsWithOperator, fmt.Sprintf("key may not begin with %q", key[0]))
	case '!':
		return forbidden(line, col, CodeExplicitTag, "tags (!x / !!x) are not allowed in SYON")
	case '&':
		return forbidden(line, col, CodeAnchor, "anchors (&x) are not allowed in SYON")
	case '*':
		return forbidden(line, col, CodeAlias, "aliases (*x) are not allowed in SYON")
	case '?':
		return forbidden(line, col, CodeComplexKey, "complex keys (?) are not allowed in SYON")
	}
	return nil
}

func checkForbiddenValue(v string, line, col int) error {
	if v == "" {
		return nil
	}
	switch v[0] {
	case '{':
		return forbidden(line, col, CodeFlowMapping, "flow mappings ({…}) are not allowed in SYON")
	case '[':
		if strings.HasPrefix(v, "[[[") {
			// Name the removed construct rather than reporting a generic
			// flow-sequence error, which would not say what to write instead.
			return forbidden(line, col, CodeLiteralBlockRemoved,
				"`[[[ … ]]]` literal blocks were removed; use a `|` block scalar instead")
		}
		return forbidden(line, col, CodeFlowSequence, "flow sequences ([…]) are not allowed in SYON")
	case '!':
		return forbidden(line, col, CodeExplicitTag, "tags (!x / !!x) are not allowed in SYON")
	case '&':
		return forbidden(line, col, CodeAnchor, "anchors (&x) are not allowed in SYON")
	case '*':
		return forbidden(line, col, CodeAlias, "aliases (*x) are not allowed in SYON")
	}
	return nil
}

// stripTrailingComment removes a structural trailing comment (" # …" or " #").
func stripTrailingComment(s string) string {
	for i := 0; i+1 < len(s); i++ {
		if s[i] == ' ' && s[i+1] == '#' && (i+2 == len(s) || s[i+2] == ' ') {
			return strings.TrimRight(s[:i], " ")
		}
	}
	return strings.TrimRight(s, " ")
}

func unquote(s string) string {
	inner := s[1 : len(s)-1]
	var b strings.Builder
	for i := 0; i < len(inner); i++ {
		if inner[i] == '\\' && i+1 < len(inner) {
			i++
			switch inner[i] {
			case 'n':
				b.WriteByte('\n')
			case 't':
				b.WriteByte('\t')
			default:
				b.WriteByte(inner[i])
			}
			continue
		}
		b.WriteByte(inner[i])
	}
	return b.String()
}

// dedent removes the common leading-space prefix from non-blank lines and trims
// surrounding blank lines.
func dedent(lines []string) string {
	min := -1
	for _, l := range lines {
		if strings.TrimSpace(l) == "" {
			continue
		}
		n := 0
		for n < len(l) && l[n] == ' ' {
			n++
		}
		if min < 0 || n < min {
			min = n
		}
	}
	if min < 0 {
		min = 0
	}
	out := make([]string, len(lines))
	for i, l := range lines {
		if len(l) >= min {
			out[i] = l[min:]
		} else {
			out[i] = strings.TrimLeft(l, " ")
		}
	}
	for len(out) > 0 && strings.TrimSpace(out[0]) == "" {
		out = out[1:]
	}
	for len(out) > 0 && strings.TrimSpace(out[len(out)-1]) == "" {
		out = out[:len(out)-1]
	}
	return strings.Join(out, "\n")
}
