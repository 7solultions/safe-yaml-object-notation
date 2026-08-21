package syon_test

import (
	"reflect"
	"strings"
	"testing"

	syon "github.com/object-notation-environment/safe-yaml-object-notation/syon-go"
)

type Item struct {
	ID          string   `syon:"id"`
	Name        string   `syon:"name"`
	Description string   `syon:"description"`
	Takeable    bool     `syon:"takeable"`
	Contexts    []string `syon:"contexts"`
}

const itemDoc = `# an item
id: helm
name: Helm mit Stirnlampe
description: |-
  A robust helm.
  Its lamp flickers.
takeable: true      # trailing comment
contexts:
  - light
  - wearable
`

func TestUnmarshalItem(t *testing.T) {
	var it Item
	if err := syon.Unmarshal([]byte(itemDoc), &it); err != nil {
		t.Fatalf("Unmarshal: %v", err)
	}
	if it.ID != "helm" || it.Name != "Helm mit Stirnlampe" {
		t.Errorf("scalars wrong: %+v", it)
	}
	if it.Description != "A robust helm.\nIts lamp flickers." {
		t.Errorf("block scalar not dedented/joined: %q", it.Description)
	}
	if it.Takeable != true {
		t.Errorf("bool coercion failed")
	}
	if !reflect.DeepEqual(it.Contexts, []string{"light", "wearable"}) {
		t.Errorf("sequence wrong: %v", it.Contexts)
	}
}

type Room struct {
	ID    string `syon:"id"`
	Title string `syon:"title"`
}
type Exit struct {
	To     string `syon:"to"`
	Puzzle string `syon:"puzzle"`
}
type World struct {
	Rooms []Room          `syon:"rooms"`
	Exits map[string]Exit `syon:"exits"`
}

const worldDoc = `rooms:
  -
    id: tor
    title: Das Tor
  -
    id: halle
    title: Die Halle
exits:
  north:
    to: halle
  west:
    to: archiv
    puzzle: repo-tor
`

func TestUnmarshalNested(t *testing.T) {
	var w World
	if err := syon.Unmarshal([]byte(worldDoc), &w); err != nil {
		t.Fatalf("Unmarshal: %v", err)
	}
	if len(w.Rooms) != 2 || w.Rooms[0].ID != "tor" || w.Rooms[1].Title != "Die Halle" {
		t.Errorf("sequence of mappings wrong: %+v", w.Rooms)
	}
	if w.Exits["west"].To != "archiv" || w.Exits["west"].Puzzle != "repo-tor" {
		t.Errorf("nested mapping wrong: %+v", w.Exits)
	}
}

func TestNoImplicitTyping(t *testing.T) {
	// "true" into a string field stays the string "true"; digits stay strings.
	var s struct {
		Flag string `syon:"flag"`
		Num  string `syon:"num"`
	}
	if err := syon.Unmarshal([]byte("flag: true\nnum: 007\n"), &s); err != nil {
		t.Fatal(err)
	}
	if s.Flag != "true" || s.Num != "007" {
		t.Errorf("implicit typing leaked: %+v", s)
	}
}

func TestForbiddenAndSyntax(t *testing.T) {
	cases := []struct {
		name, src, kind string
	}{
		{"doc-start", "---\nkey: v\n", "forbidden"},
		{"anchor", "key: &a value\n", "forbidden"},
		{"alias", "key: *a\n", "forbidden"},
		{"tag", "key: !!str x\n", "forbidden"},
		{"flow-seq", "key: [1, 2]\n", "forbidden"},
		{"flow-map", "key: {a: b}\n", "forbidden"},
		{"tab-indent", "key:\n\tnested: v\n", "syntax"},
		{"dup-key", "a: 1\na: 2\n", "syntax"},
		{"bracketed-literal-removed", "d: [[[\nhello\n]]]\n", "forbidden"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			var v map[string]any
			err := syon.Unmarshal([]byte(c.src), &v)
			if err == nil {
				t.Fatalf("expected error, got none")
			}
			se, ok := err.(*syon.Error)
			if !ok {
				t.Fatalf("expected *syon.Error, got %T: %v", err, err)
			}
			if se.Kind != c.kind {
				t.Errorf("kind = %q, want %q (%v)", se.Kind, c.kind, se)
			}
		})
	}
}

func TestRemovedLiteralBlockIsNamedNotReportedAsFlow(t *testing.T) {
	// `[[[` starts with '[', so the generic flow-sequence error would fire
	// and never say what to write instead. It must be named.
	var v map[string]any
	err := syon.Unmarshal([]byte("d: [[[\n  ok\n]]]\n"), &v)
	if err == nil {
		t.Fatal("expected `[[[` to be rejected")
	}
	if !strings.Contains(err.Error(), "block scalar") {
		t.Errorf("error does not name the replacement: %v", err)
	}
}

func TestBlockScalarValue(t *testing.T) {
	var v map[string]any
	if err := syon.Unmarshal([]byte("d: |\n  ok\n"), &v); err != nil {
		t.Fatalf("block scalar rejected: %v", err)
	}
	if v["d"] != "ok\n" {
		t.Errorf("block scalar value = %q, want %q", v["d"], "ok\n")
	}
}

func TestValuesWithColonsAndDashes(t *testing.T) {
	// The spacing rule: `:` and `-` inside values need no quoting.
	var v map[string]any
	src := "url: https://example.com\ntag: -draft\nid: abc#123\n"
	if err := syon.Unmarshal([]byte(src), &v); err != nil {
		t.Fatal(err)
	}
	if v["url"] != "https://example.com" || v["tag"] != "-draft" || v["id"] != "abc#123" {
		t.Errorf("spacing rule mishandled: %v", v)
	}
}

func TestOnlyFirstColonSpaceOnALineIsStructural(t *testing.T) {
	// Only the FIRST `: ` on a line separates key from value; every later
	// colon, even a `: `-shaped one, is ordinary value text.
	var v map[string]any
	src := "key: value: with colon: multiple times\n"
	if err := syon.Unmarshal([]byte(src), &v); err != nil {
		t.Fatal(err)
	}
	want := "value: with colon: multiple times"
	if v["key"] != want {
		t.Errorf("key = %q, want %q", v["key"], want)
	}
}

func TestDashIsStructuralOnlyAsFirstNonSpaceCharOfTheLine(t *testing.T) {
	// A `-` later in the line -- even followed by a space, even preceded by
	// a space -- is NOT a sequence-item marker unless it is the first
	// non-space character on the line.
	var v map[string]any
	if err := syon.Unmarshal([]byte("note: this - is not a list item\n"), &v); err != nil {
		t.Fatal(err)
	}
	want := "this - is not a list item"
	if v["note"] != want {
		t.Errorf("note = %q, want %q", v["note"], want)
	}

	// A `-` inside a key (not preceded by whitespace at all) is also just
	// ordinary key text.
	v = nil
	if err := syon.Unmarshal([]byte("a-b: value\n"), &v); err != nil {
		t.Fatal(err)
	}
	if _, ok := v["a-b"]; !ok {
		t.Errorf("expected key %q, got %v", "a-b", v)
	}
}

func TestGenericInterface(t *testing.T) {
	var v any
	if err := syon.Unmarshal([]byte("a: 1\nb:\n  - x\n  - y\n"), &v); err != nil {
		t.Fatal(err)
	}
	m := v.(map[string]any)
	if m["a"] != "1" {
		t.Errorf("a = %v", m["a"])
	}
	if !reflect.DeepEqual(m["b"], []any{"x", "y"}) {
		t.Errorf("b = %v", m["b"])
	}
}

// yaml-tag fallback eases migration from gopkg.in/yaml.v3.
func TestYAMLTagFallback(t *testing.T) {
	var s struct {
		Name string `yaml:"name"`
	}
	if err := syon.Unmarshal([]byte("name: Hans\n"), &s); err != nil {
		t.Fatal(err)
	}
	if s.Name != "Hans" {
		t.Errorf("yaml tag fallback failed: %+v", s)
	}
}

func TestParseTree(t *testing.T) {
	n, err := syon.Parse([]byte("k: v\n"))
	if err != nil {
		t.Fatal(err)
	}
	if n.Kind != syon.MappingNode || n.Map["k"].Str != "v" {
		t.Errorf("tree wrong: %+v", n)
	}
	if !strings.Contains(n.Map["k"].Str, "v") {
		t.Errorf("scalar missing")
	}
}
