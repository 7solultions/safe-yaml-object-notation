use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use syon_parser::{parse, parse_document, Phase1Counts, Value};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && args[1] == "phase1" {
        run_phase1(&args[2..]);
        return;
    }

    if args.len() != 2 {
        eprintln!("Usage: syon <file.syon>");
        eprintln!("       syon phase1 [FILE...]");
        process::exit(1);
    }

    let src = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("error reading {}: {e}", args[1]);
        process::exit(1);
    });

    match parse_document(&src) {
        Ok(doc) => println!("{}", to_json(&doc.body, 0)),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn to_json(value: &Value, depth: usize) -> String {
    let pad = "  ".repeat(depth);
    let inner = "  ".repeat(depth + 1);
    match value {
        Value::Scalar(s) => serde_json::to_string(s).unwrap(),
        Value::LiteralBlock(s) => serde_json::to_string(s).unwrap(),
        Value::Mapping(entries) => {
            if entries.is_empty() {
                return "{}".into();
            }
            let pairs: Vec<String> = entries
                .iter()
                .map(|e| {
                    format!(
                        "{inner}{}: {}",
                        serde_json::to_string(&e.key).unwrap(),
                        to_json(&e.value, depth + 1)
                    )
                })
                .collect();
            format!("{{\n{}\n{pad}}}", pairs.join(",\n"))
        }
        Value::Sequence(items) => {
            if items.is_empty() {
                return "[]".into();
            }
            let elems: Vec<String> = items
                .iter()
                .map(|item| format!("{inner}{}", to_json(&item.value, depth + 1)))
                .collect();
            format!("[\n{}\n{pad}]", elems.join(",\n"))
        }
    }
}

// ---------------------------------------------------------------------------
// `syon phase1` — evaluate block usage, complexity, and YAML compatibility
// across one or more SYON files.
//
// The report names its sections rather than numbering them. It used to say
// "block2"/"block3" with the opposite meaning to spec/02-grammar.md; removing
// `[[[ ... ]]]` left nothing for the two numberings to disagree about, and
// self-describing keys keep it that way. See
// docs/decisions/0006-phase1-block-numbering.syon.
// ---------------------------------------------------------------------------

fn run_phase1(file_args: &[String]) {
    let files: Vec<PathBuf> = if file_args.is_empty() {
        default_phase1_corpus()
    } else {
        file_args.iter().map(PathBuf::from).collect()
    };

    if files.is_empty() {
        eprintln!("phase1: no .syon files found to analyze (pass file paths explicitly, or run from a directory containing examples/ or docs/decisions/)");
        process::exit(1);
    }

    let mut entries: Vec<(String, Phase1Counts)> = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error reading {}: {e}", path.display());
            process::exit(1);
        });
        match parse(&src) {
            Ok(file) => {
                let mut counts = Phase1Counts::default();
                counts.add_file(&file);
                entries.push((path.display().to_string(), counts));
            }
            Err(e) => {
                eprintln!("error parsing {}: {e}", path.display());
                process::exit(1);
            }
        }
    }

    let report = render_phase1_report(&entries);
    let out_path = "phase1.report.syon";
    fs::write(out_path, &report).unwrap_or_else(|e| {
        eprintln!("error writing {out_path}: {e}");
        process::exit(1);
    });
    println!("wrote {out_path} ({} file(s) analyzed)", entries.len());
}

fn default_phase1_corpus() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_syon_files(Path::new("examples"), &mut files);
    collect_syon_files(Path::new("docs/decisions"), &mut files);
    files.sort();
    files
}

fn collect_syon_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    children.sort();
    for path in children {
        if path.is_dir() {
            collect_syon_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("syon") {
            out.push(path);
        }
    }
}

/// Double-quote a string for use as a SYON scalar, escaping `"` and `\`.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

fn render_phase1_report(entries: &[(String, Phase1Counts)]) -> String {
    let mut out = String::new();
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    out.push_str("phase1-report:\n");
    out.push_str(&format!("  generated-at-unix: {generated_at_unix}\n"));
    out.push_str("  files:\n");

    for (path, c) in entries {
        out.push_str("    -\n");
        out.push_str(&format!("      path: {}\n", quote(path)));
        out.push_str("      block1:\n");
        out.push_str("        structural:\n");
        out.push_str(&format!("          colon-space: {}\n", c.mapping_entries));
        out.push_str(&format!("          dash-space: {}\n", c.sequence_items));
        out.push_str(&format!("          hash-space: {}\n", c.comments));
        out.push_str("        inline:\n");
        out.push_str(&format!("          colon: {}\n", c.inline_colon));
        out.push_str(&format!("          dash: {}\n", c.inline_dash));
        out.push_str(&format!("          hash: {}\n", c.inline_hash));
        out.push_str("      block-scalars:\n");
        out.push_str(&format!("        count: {}\n", c.literal_blocks));
        out.push_str("      fences:\n");
        out.push_str(&format!("        count: {}\n", c.fences));
        if !c.fence_formats.is_empty() {
            out.push_str("        formats:\n");
            for f in &c.fence_formats {
                out.push_str(&format!("          - {}\n", quote(f)));
            }
        }
        if !c.fence_paths.is_empty() {
            out.push_str("        paths:\n");
            for p in &c.fence_paths {
                out.push_str(&format!("          - {}\n", quote(p)));
            }
        }
        out.push_str(&format!("      complexity: {}\n", c.complexity()));
        out.push_str(&format!("      yaml-compatible: {}\n", c.yaml_compatible()));
        out.push_str(&format!(
            "      yaml-compatibility-percent: {}\n",
            c.yaml_compatibility_percent()
        ));
    }

    let files_analyzed = entries.len() as u64;
    let total_complexity: u64 = entries.iter().map(|(_, c)| c.complexity()).sum();
    let average_complexity = total_complexity.checked_div(files_analyzed).unwrap_or(0);
    let yaml_compatible_files = entries.iter().filter(|(_, c)| c.yaml_compatible()).count() as u64;
    let yaml_incompatible_files = files_analyzed - yaml_compatible_files;

    let sum_mapping: u64 = entries.iter().map(|(_, c)| c.mapping_entries).sum();
    let sum_sequence: u64 = entries.iter().map(|(_, c)| c.sequence_items).sum();
    let sum_literal: u64 = entries.iter().map(|(_, c)| c.literal_blocks).sum();
    let sum_fences: u64 = entries.iter().map(|(_, c)| c.fences).sum();
    // Block scalars count as compatible alongside mappings and sequences --
    // they are ordinary YAML 1.2. Only a fence costs compatibility. Keep this
    // in step with Phase1Counts::yaml_compatibility_percent.
    let compatible_total = sum_mapping + sum_sequence + sum_literal;
    let grand_total = compatible_total + sum_fences;
    let overall_percent = (100 * compatible_total + grand_total / 2)
        .checked_div(grand_total)
        .unwrap_or(100);

    out.push_str("  summary:\n");
    out.push_str(&format!("    files-analyzed: {files_analyzed}\n"));
    out.push_str(&format!("    total-complexity: {total_complexity}\n"));
    out.push_str(&format!("    average-complexity: {average_complexity}\n"));
    out.push_str(&format!("    yaml-compatible-files: {yaml_compatible_files}\n"));
    out.push_str(&format!(
        "    yaml-incompatible-files: {yaml_incompatible_files}\n"
    ));
    out.push_str(&format!(
        "    overall-yaml-compatibility-percent: {overall_percent}\n"
    ));

    out
}
