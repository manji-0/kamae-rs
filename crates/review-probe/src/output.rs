use crate::types::{ProbeOutput, PublicItem};

pub fn render_text(output: &ProbeOutput, limit: usize) -> String {
    let limit = limit.max(1);
    let mut lines = vec![
        "# kamae-rs Review Probe".to_string(),
        String::new(),
        "These are review leads, not findings. Confirm reachability, invariants, and project conventions before reporting.".to_string(),
        String::new(),
        format!("Scanned Rust files: {}", output.scanned_files.len()),
    ];

    if !output.suggested_commands.is_empty() {
        lines.push(String::new());
        lines.push("## Suggested Commands".to_string());
        for command in &output.suggested_commands {
            lines.push(format!("- `{command}`"));
        }
    }

    lines.push(String::new());
    lines.push("## Summary".to_string());
    for (key, value) in [
        (
            "leads_by_category",
            format!("{:?}", output.summary.leads_by_category),
        ),
        (
            "leads_by_checklist",
            format!("{:?}", output.summary.leads_by_checklist),
        ),
        ("public_items", output.summary.public_items.to_string()),
        (
            "public_items_missing_docs",
            output.summary.public_items_missing_docs.to_string(),
        ),
        (
            "unsafe_public_items_missing_safety",
            output
                .summary
                .unsafe_public_items_missing_safety
                .to_string(),
        ),
        (
            "documented_result_items_missing_errors",
            output
                .summary
                .documented_result_items_missing_errors
                .to_string(),
        ),
    ] {
        lines.push(format!("- {key}: {value}"));
    }

    lines.push(String::new());
    lines.push("## Leads".to_string());
    if output.leads.is_empty() {
        lines.push("- No pattern leads found.".to_string());
    } else {
        for lead in output.leads.iter().take(limit) {
            lines.push(format!(
                "- `{}:{}` [{} -> {}] {}\n  {}",
                lead.path, lead.line, lead.category, lead.checklist, lead.snippet, lead.note
            ));
        }
        if output.leads.len() > limit {
            lines.push(format!(
                "- Truncated {} additional lead(s). Use `--limit` to show more.",
                output.leads.len() - limit
            ));
        }
    }

    let missing_docs: Vec<&PublicItem> = output
        .public_items
        .iter()
        .filter(|item| !item.has_doc)
        .collect();
    let result_without_errors: Vec<&PublicItem> = output
        .public_items
        .iter()
        .filter(|item| item.signature.contains("Result") && item.has_doc && !item.has_errors)
        .collect();
    let unsafe_without_safety: Vec<&PublicItem> = output
        .public_items
        .iter()
        .filter(|item| item.signature.contains("unsafe ") && !item.has_safety)
        .collect();

    lines.push(String::new());
    lines.push("## Rustdoc Leads".to_string());
    for (title, items, note) in [
        (
            "Public items missing rustdoc",
            missing_docs,
            "Review whether this public API needs a domain contract.",
        ),
        (
            "Documented Result items missing # Errors",
            result_without_errors,
            "Callers may need error semantics.",
        ),
        (
            "Unsafe public items missing # Safety",
            unsafe_without_safety,
            "Unsafe caller obligations must be explicit.",
        ),
    ] {
        lines.push(format!("### {title}"));
        if items.is_empty() {
            lines.push("- None.".to_string());
        } else {
            let total = items.len();
            for item in items.into_iter().take(limit) {
                lines.push(format!(
                    "- `{}:{}` {}\n  {}",
                    item.path, item.line, item.signature, note
                ));
            }
            if total > limit {
                lines.push(format!("- Truncated {} additional item(s).", total - limit));
            }
        }
    }

    lines.join("\n")
}

pub fn render_json(output: &ProbeOutput) -> String {
    serde_json::to_string_pretty(output).expect("probe output serializes")
}
