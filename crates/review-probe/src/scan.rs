use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".dagayn",
    "target",
    "node_modules",
    ".venv",
    "venv",
];

pub fn iter_rust_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path.clone());
            }
            continue;
        }
        if path.is_dir() {
            collect_rust_files(path, &mut files);
        }
    }
    files.sort();
    files.dedup();
    files
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| SKIP_DIRS.contains(&name))
            {
                continue;
            }
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

pub fn rel_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn is_generated(path: &Path, text: &str) -> bool {
    let rel = path.to_string_lossy().to_ascii_lowercase();
    let head = text.lines().take(8).collect::<Vec<_>>().join("\n").to_ascii_lowercase();
    rel.contains("generated")
        || rel.contains("bindings")
        || head.contains("@generated")
        || head.contains("automatically generated")
        || head.contains("do not edit")
}

pub fn snippet_at(text: &str, line: usize) -> String {
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .chars()
        .take(160)
        .collect()
}

pub fn line_of(span: proc_macro2::Span) -> usize {
    span.start().line
}

pub fn truncate(mut value: String, max: usize) -> String {
    if value.chars().count() > max {
        value = value.chars().take(max).collect();
    }
    value
}
