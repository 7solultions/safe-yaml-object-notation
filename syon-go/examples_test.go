package syon

import (
	"os"
	"path/filepath"
	"testing"
)

// parseAllSyonUnder parses every .syon file found under root, failing the
// test for any file that doesn't parse.
func parseAllSyonUnder(t *testing.T, root string) {
	t.Helper()
	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() || filepath.Ext(path) != ".syon" {
			return nil
		}
		t.Run(path, func(t *testing.T) {
			data, err := os.ReadFile(path)
			if err != nil {
				t.Fatalf("read: %v", err)
			}
			if _, err := Parse(data); err != nil {
				t.Fatalf("parse: %v", err)
			}
		})
		return nil
	})
	if err != nil {
		t.Fatalf("walk %s: %v", root, err)
	}
}

// TestExamplesParse parses every .syon file under ../examples to keep the
// canonical examples valid against this implementation too, alongside the
// Rust one (see examples-valid in .github/workflows/ci.yml).
func TestExamplesParse(t *testing.T) {
	parseAllSyonUnder(t, filepath.Join("..", "examples"))
}

// TestDecisionsParse parses every ADR under ../design/architecture — the ADR
// log is itself a corpus of real SYON documents (see
// design/architecture/0001-*.syon).
func TestDecisionsParse(t *testing.T) {
	parseAllSyonUnder(t, filepath.Join("..", "design", "architecture"))
}
