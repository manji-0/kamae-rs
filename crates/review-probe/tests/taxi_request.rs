use std::path::PathBuf;

use kamae_review_probe::probe::probe_paths;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn taxi_request_emits_expected_lead_categories() {
    let root = repo_root();
    let example = root.join("skills/kamae-rs/examples/taxi-request.rs");
    let output = probe_paths(&root, &[example]);

    assert_eq!(output.scanned_files.len(), 1);
    assert!(output
        .summary
        .leads_by_category
        .contains_key("lint-suppression"));
    assert!(output.summary.leads_by_category.contains_key("panic-path"));
    assert!(output.summary.public_items >= 10);
    assert_eq!(
        output.summary.public_items_missing_docs,
        output.summary.public_items
    );
}

#[test]
fn json_output_is_stable_shape() {
    let root = repo_root();
    let example = root.join("skills/kamae-rs/examples/taxi-request.rs");
    let output = probe_paths(&root, &[example]);
    let json = serde_json::to_value(&output).expect("serialize probe output");

    assert!(json.get("scanned_files").is_some());
    assert!(json.get("summary").is_some());
    assert!(json.get("suggested_commands").is_some());
    assert!(json.get("leads").is_some());
    assert!(json.get("public_items").is_some());
}
