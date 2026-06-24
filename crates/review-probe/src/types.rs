use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Lead {
    pub category: &'static str,
    pub checklist: &'static str,
    pub path: String,
    pub line: usize,
    pub snippet: String,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicItem {
    pub path: String,
    pub line: usize,
    pub signature: String,
    pub has_doc: bool,
    pub has_errors: bool,
    pub has_panics: bool,
    pub has_safety: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub leads_by_category: std::collections::BTreeMap<String, usize>,
    pub leads_by_checklist: std::collections::BTreeMap<String, usize>,
    pub public_items: usize,
    pub public_items_missing_docs: usize,
    pub unsafe_public_items_missing_safety: usize,
    pub documented_result_items_missing_errors: usize,
}

#[derive(Debug, Serialize)]
pub struct ProbeOutput {
    pub scanned_files: Vec<String>,
    pub summary: Summary,
    pub suggested_commands: Vec<String>,
    pub leads: Vec<Lead>,
    pub public_items: Vec<PublicItem>,
}

pub struct Pattern {
    pub category: &'static str,
    pub checklist: &'static str,
    pub note: &'static str,
}

impl Pattern {
    pub const fn new(category: &'static str, checklist: &'static str, note: &'static str) -> Self {
        Self {
            category,
            checklist,
            note,
        }
    }
}
