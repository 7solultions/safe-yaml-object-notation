package syon_test

import (
	"reflect"
	"strings"
	"testing"

	syon "github.com/object-notation-environment/safe-yaml-object-notation/syon-go"
)

func TestMarshalRoundTripItem(t *testing.T) {
	in := Item{
		ID:          "helm",
		Name:        "Helm mit Stirnlampe",
		Description: "A robust helm.\nIts lamp flickers.",
		Takeable:    true,
		Contexts:    []string{"light", "wearable"},
	}
	data, err := syon.Marshal(in)
	if err != nil {
		t.Fatalf("Marshal: %v", err)
	}
	var out Item
	if err := syon.Unmarshal(data, &out); err != nil {
		t.Fatalf("Unmarshal(Marshal): %v\n---\n%s", err, data)
	}
	if !reflect.DeepEqual(in, out) {
		t.Errorf("round-trip mismatch:\n in=%+v\nout=%+v\n---\n%s", in, out, data)
	}
	// The multi-line field must be written as a block scalar.
	if !strings.Contains(string(data), "description: |") {
		t.Errorf("multiline not written as a block scalar:\n%s", data)
	}
}

func TestMarshalRoundTripNested(t *testing.T) {
	in := World{
		Rooms: []Room{{ID: "tor", Title: "Das Tor"}, {ID: "halle", Title: "Die Halle"}},
		Exits: map[string]Exit{"west": {To: "archiv", Puzzle: "repo-tor"}},
	}
	data, err := syon.Marshal(in)
	if err != nil {
		t.Fatal(err)
	}
	var out World
	if err := syon.Unmarshal(data, &out); err != nil {
		t.Fatalf("%v\n---\n%s", err, data)
	}
	if !reflect.DeepEqual(in, out) {
		t.Errorf("round-trip mismatch:\n in=%+v\nout=%+v\n---\n%s", in, out, data)
	}
}

func TestMarshalEmptyListRoundTrips(t *testing.T) {
	type Save struct {
		Inventory []string `syon:"inventory"`
		Location  string   `syon:"location"`
	}
	in := Save{Inventory: []string{}, Location: "tor"}
	data, err := syon.Marshal(in)
	if err != nil {
		t.Fatal(err)
	}
	// No forbidden `[]` flow in the output.
	if strings.Contains(string(data), "[]") {
		t.Errorf("emitted forbidden flow empty list:\n%s", data)
	}
	var out Save
	if err := syon.Unmarshal(data, &out); err != nil {
		t.Fatalf("%v\n---\n%s", err, data)
	}
	if out.Location != "tor" || out.Inventory == nil || len(out.Inventory) != 0 {
		t.Errorf("empty list did not round-trip: %+v\n---\n%s", out, data)
	}
}

func TestMarshalQuotesWhenNeeded(t *testing.T) {
	type M struct {
		Empty   string `syon:"empty"`
		Hashish string `syon:"hashish"`
		Bracket string `syon:"bracket"`
		Plain   string `syon:"plain"`
	}
	in := M{Empty: "", Hashish: "a # b", Bracket: "[x]", Plain: "https://ok.com"}
	data, _ := syon.Marshal(in)
	s := string(data)
	if !strings.Contains(s, `empty: ""`) {
		t.Errorf("empty not quoted:\n%s", s)
	}
	if !strings.Contains(s, `hashish: "a # b"`) {
		t.Errorf("inline-comment-ish not quoted:\n%s", s)
	}
	if !strings.Contains(s, `bracket: "[x]"`) {
		t.Errorf("flow-ish not quoted:\n%s", s)
	}
	if !strings.Contains(s, "plain: https://ok.com") {
		t.Errorf("plain over-quoted:\n%s", s)
	}
	var out M
	if err := syon.Unmarshal(data, &out); err != nil {
		t.Fatalf("%v\n---\n%s", err, data)
	}
	if out != in {
		t.Errorf("quote round-trip mismatch:\n in=%+v\nout=%+v", in, out)
	}
}
