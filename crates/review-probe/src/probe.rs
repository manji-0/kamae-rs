use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::scan::{iter_rust_files, rel_path};
use crate::types::{Lead, ProbeOutput, PublicItem, Summary};
use crate::visitor::{collect_file, CollectOutcome};

pub fn probe_paths(root: &Path, paths: &[PathBuf]) -> ProbeOutput {
    let files = iter_rust_files(paths);
    let mut leads = Vec::new();
    let mut public_items = Vec::new();
    let scanned_files = files
        .iter()
        .map(|path| rel_path(path, root))
        .collect::<Vec<_>>();

    for path in &files {
        let rel = rel_path(path, root);
        match collect_file(path, rel.clone()) {
            Ok(CollectOutcome::Collected {
                leads: file_leads,
                public_items: file_items,
            }) => {
                leads.extend(file_leads);
                public_items.extend(file_items);
            }
            Ok(CollectOutcome::ParseError(error)) => {
                leads.push(Lead {
                    category: "parse-error",
                    checklist: "fmt-lint.md",
                    path: rel,
                    line: error.span().start().line,
                    snippet: error.to_string(),
                    note: "File did not parse; fix syntax before review.",
                });
            }
            Ok(CollectOutcome::Skipped) => {}
            Err(error) => {
                leads.push(Lead {
                    category: "read-error",
                    checklist: "fmt-lint.md",
                    path: rel,
                    line: 1,
                    snippet: error.to_string(),
                    note: "File could not be read.",
                });
            }
        }
    }

    let suggested_commands = cargo_commands(root, &public_items);
    let summary = summarize(&leads, &public_items);

    ProbeOutput {
        scanned_files,
        summary,
        suggested_commands,
        leads,
        public_items,
    }
}

fn cargo_commands(root: &Path, public_items: &[PublicItem]) -> Vec<String> {
    if !root.join("Cargo.toml").exists() {
        return Vec::new();
    }

    let mut commands = vec![
        "cargo fmt --check".to_string(),
        "cargo clippy --all-targets --all-features".to_string(),
        "cargo test".to_string(),
    ];
    if public_items.iter().any(|item| item.has_doc) {
        commands.push("cargo doc --no-deps".to_string());
    }
    commands
}

fn summarize(leads: &[Lead], public_items: &[PublicItem]) -> Summary {
    let mut leads_by_category = BTreeMap::new();
    let mut leads_by_checklist = BTreeMap::new();
    for lead in leads {
        *leads_by_category
            .entry(lead.category.to_string())
            .or_insert(0) += 1;
        *leads_by_checklist
            .entry(lead.checklist.to_string())
            .or_insert(0) += 1;
    }

    let missing_docs = public_items.iter().filter(|item| !item.has_doc).count();
    let unsafe_without_safety = public_items
        .iter()
        .filter(|item| item.signature.contains("unsafe ") && !item.has_safety)
        .count();
    let result_without_errors = public_items
        .iter()
        .filter(|item| item.signature.contains("Result") && item.has_doc && !item.has_errors)
        .count();

    Summary {
        leads_by_category,
        leads_by_checklist,
        public_items: public_items.len(),
        public_items_missing_docs: missing_docs,
        unsafe_public_items_missing_safety: unsafe_without_safety,
        documented_result_items_missing_errors: result_without_errors,
    }
}
