// Command phase1 evaluates block usage, complexity, and YAML compatibility
// across one or more SYON files, writing phase1.report.syon.
//
// The report names its sections rather than numbering them -- see the same
// note in crates/syon-cli/src/main.rs, whose output this matches byte for
// byte.
//
// Usage (run from within syon-go/, matching `go test`'s own convention):
//
//	go run ./cmd/phase1 [FILE...]
//
// With no file arguments, it walks the default corpus: ../examples/**/*.syon
// and ../design/architecture/*.syon -- the repo root's examples/ and
// design/architecture/, one level up from the syon-go module.
package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"

	syon "github.com/object-notation-environment/safe-yaml-object-notation/syon-go"
)

type fileReport struct {
	path   string
	counts *syon.Phase1Counts
}

func main() {
	files := os.Args[1:]
	var paths []string
	if len(files) == 0 {
		paths = defaultCorpus()
	} else {
		paths = files
	}

	if len(paths) == 0 {
		fmt.Fprintln(os.Stderr, "phase1: no .syon files found to analyze (pass file paths explicitly, or run from a directory containing examples/ or design/architecture/)")
		os.Exit(1)
	}

	reports := make([]fileReport, 0, len(paths))
	for _, p := range paths {
		data, err := os.ReadFile(p)
		if err != nil {
			fmt.Fprintf(os.Stderr, "error reading %s: %v\n", p, err)
			os.Exit(1)
		}
		node, err := syon.Parse(data)
		if err != nil {
			fmt.Fprintf(os.Stderr, "error parsing %s: %v\n", p, err)
			os.Exit(1)
		}
		c := &syon.Phase1Counts{}
		c.AddNode(node)
		reports = append(reports, fileReport{path: p, counts: c})
	}

	report := renderReport(reports)
	const outPath = "phase1.report.syon"
	if err := os.WriteFile(outPath, []byte(report), 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "error writing %s: %v\n", outPath, err)
		os.Exit(1)
	}
	fmt.Printf("wrote %s (%d file(s) analyzed)\n", outPath, len(reports))
}

func defaultCorpus() []string {
	var out []string
	for _, root := range []string{
		filepath.Join("..", "examples"),
		filepath.Join("..", "design", "architecture"),
	} {
		_ = filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
			if err != nil || d == nil {
				return nil //nolint:nilerr // missing root dirs are not fatal
			}
			if !d.IsDir() && filepath.Ext(path) == ".syon" {
				out = append(out, path)
			}
			return nil
		})
	}
	sort.Strings(out)
	return out
}

func quote(s string) string {
	var b strings.Builder
	b.WriteByte('"')
	for _, ch := range s {
		if ch == '"' || ch == '\\' {
			b.WriteByte('\\')
		}
		b.WriteRune(ch)
	}
	b.WriteByte('"')
	return b.String()
}

func renderReport(reports []fileReport) string {
	var b strings.Builder
	b.WriteString("phase1-report:\n")
	fmt.Fprintf(&b, "  generated-at-unix: %d\n", time.Now().Unix())
	b.WriteString("  files:\n")

	for _, r := range reports {
		c := r.counts
		b.WriteString("    -\n")
		fmt.Fprintf(&b, "      path: %s\n", quote(r.path))
		b.WriteString("      block1:\n")
		b.WriteString("        structural:\n")
		fmt.Fprintf(&b, "          colon-space: %d\n", c.MappingEntries)
		fmt.Fprintf(&b, "          dash-space: %d\n", c.SequenceItems)
		fmt.Fprintf(&b, "          hash-space: %d\n", c.Comments)
		b.WriteString("        inline:\n")
		fmt.Fprintf(&b, "          colon: %d\n", c.InlineColon)
		fmt.Fprintf(&b, "          dash: %d\n", c.InlineDash)
		fmt.Fprintf(&b, "          hash: %d\n", c.InlineHash)
		b.WriteString("      block-scalars:\n")
		fmt.Fprintf(&b, "        count: %d\n", c.LiteralBlocks)
		b.WriteString("      fences:\n")
		fmt.Fprintf(&b, "        count: %d\n", c.Fences)
		if len(c.FenceFormats) > 0 {
			b.WriteString("        formats:\n")
			for _, f := range c.FenceFormats {
				fmt.Fprintf(&b, "          - %s\n", quote(f))
			}
		}
		if len(c.FencePaths) > 0 {
			b.WriteString("        paths:\n")
			for _, p := range c.FencePaths {
				fmt.Fprintf(&b, "          - %s\n", quote(p))
			}
		}
		fmt.Fprintf(&b, "      complexity: %d\n", c.Complexity())
		fmt.Fprintf(&b, "      yaml-compatible: %s\n", strconv.FormatBool(c.YAMLCompatible()))
		fmt.Fprintf(&b, "      yaml-compatibility-percent: %d\n", c.YAMLCompatibilityPercent())
	}

	var filesAnalyzed, totalComplexity, yamlCompatibleFiles uint64
	var sumMapping, sumSequence, sumLiteral, sumFences uint64
	for _, r := range reports {
		filesAnalyzed++
		totalComplexity += r.counts.Complexity()
		if r.counts.YAMLCompatible() {
			yamlCompatibleFiles++
		}
		sumMapping += r.counts.MappingEntries
		sumSequence += r.counts.SequenceItems
		sumLiteral += r.counts.LiteralBlocks
		sumFences += r.counts.Fences
	}
	averageComplexity := uint64(0)
	if filesAnalyzed > 0 {
		averageComplexity = totalComplexity / filesAnalyzed
	}
	// Block scalars count as compatible alongside mappings and sequences --
	// they are ordinary YAML 1.2. Only a fence costs compatibility. Keep this
	// in step with Phase1Counts.YAMLCompatibilityPercent.
	compatibleTotal := sumMapping + sumSequence + sumLiteral
	grandTotal := compatibleTotal + sumFences
	overallPercent := uint64(100)
	if grandTotal > 0 {
		overallPercent = (100*compatibleTotal + grandTotal/2) / grandTotal
	}

	b.WriteString("  summary:\n")
	fmt.Fprintf(&b, "    files-analyzed: %d\n", filesAnalyzed)
	fmt.Fprintf(&b, "    total-complexity: %d\n", totalComplexity)
	fmt.Fprintf(&b, "    average-complexity: %d\n", averageComplexity)
	fmt.Fprintf(&b, "    yaml-compatible-files: %d\n", yamlCompatibleFiles)
	fmt.Fprintf(&b, "    yaml-incompatible-files: %d\n", filesAnalyzed-yamlCompatibleFiles)
	fmt.Fprintf(&b, "    overall-yaml-compatibility-percent: %d\n", overallPercent)

	return b.String()
}
