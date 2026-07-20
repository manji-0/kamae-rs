use std::collections::BTreeSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprAwait, ExprCall, ExprMacro, ExprMethodCall, ExprUnsafe, Item, ItemFn,
    ItemMacro, Macro, Meta, TypePath, TypePtr, Visibility,
};

use crate::scan::{line_of, snippet_at, truncate};
use crate::types::{Lead, Pattern, PublicItem};

pub struct ProbeVisitor<'a> {
    pub path: String,
    pub source: &'a str,
    pub leads: Vec<Lead>,
    pub public_items: Vec<PublicItem>,
    seen_leads: BTreeSet<(String, usize)>,
}

impl<'a> ProbeVisitor<'a> {
    pub fn new(path: String, source: &'a str) -> Self {
        Self {
            path,
            source,
            leads: Vec::new(),
            public_items: Vec::new(),
            seen_leads: BTreeSet::new(),
        }
    }

    fn push_lead(&mut self, pattern: Pattern, line: usize, snippet: String) {
        let key = (pattern.category.to_string(), line);
        if !self.seen_leads.insert(key) {
            return;
        }
        self.leads.push(Lead {
            category: pattern.category,
            checklist: pattern.checklist,
            path: self.path.clone(),
            line,
            snippet: truncate(snippet, 160),
            note: pattern.note,
        });
    }

    fn push_lead_span(&mut self, pattern: Pattern, span: proc_macro2::Span, snippet: String) {
        self.push_lead(pattern, line_of(span), snippet);
    }

    fn push_ident_lead(&mut self, pattern: Pattern, ident: &syn::Ident) {
        let line = line_of(ident.span());
        let snippet = snippet_at(self.source, line);
        if snippet.is_empty() {
            self.push_lead_span(pattern, ident.span(), ident.to_string());
        } else {
            self.push_lead(pattern, line, snippet);
        }
    }

    fn visit_meta_list_tokens(&mut self, pattern: Pattern, span: proc_macro2::Span, tokens: &str) {
        let lower = tokens.to_ascii_lowercase();
        if lower.contains("deserialize")
            || lower.contains("serialize")
            || lower.contains("fromrow")
            || lower.contains("deny_unknown_fields")
            || lower.contains("serde")
        {
            self.push_lead_span(pattern, span, tokens.to_string());
        }
    }

    fn visit_error_log_tokens(&mut self, span: proc_macro2::Span, tokens: &str) {
        if tokens.contains("error = %") || tokens.contains("error.debug =") {
            let line = line_of(span);
            self.push_lead(ERROR_CHAIN_LOG, line, snippet_at(self.source, line));
        }
    }
}

const UNSAFE: Pattern = Pattern::new(
    "unsafe",
    "unsafe-boundaries.md",
    "Inspect unsafe soundness, containment behind a safe API, and safety comments.",
);

const LINT_SUPPRESSION: Pattern = Pattern::new(
    "lint-suppression",
    "fmt-lint.md",
    "Check whether lint suppression is narrow and justified.",
);

const PANIC_PATH: Pattern = Pattern::new(
    "panic-path",
    "error-handling.md",
    "Confirm this is test/startup/proven-invariant code or replace with typed errors.",
);

const BOUNDARY_DERIVE: Pattern = Pattern::new(
    "boundary-derive",
    "boundary.md",
    "Check DTO/domain separation, validation, defaults, and serialization intent.",
);

const PII_SECRET: Pattern = Pattern::new(
    "pii-secret",
    "pii-protection.md",
    "Check redaction, Debug/log exposure, and plaintext access.",
);

const PERSISTENCE_EVENT: Pattern = Pattern::new(
    "persistence-event",
    "persistence-events.md",
    "Check atomicity, idempotency, DB constraints, and event versioning.",
);

const ASYNC_OPERATIONAL: Pattern = Pattern::new(
    "async-operational",
    "persistence-events.md",
    "Check lost task failures, transaction boundaries across await, and ignored Results.",
);

const STREAM_PROJECTION: Pattern = Pattern::new(
    "stream-projection",
    "stream-continuous-queries.md",
    "Check durable cursors, backpressure, projection idempotency, and CQRS boundaries.",
);

const DOMAIN_MACRO: Pattern = Pattern::new(
    "domain-macro",
    "domain-macros.md",
    "Check generated invariants, safe Debug, and schema/version metadata.",
);

const SERVICE_BOUNDARY: Pattern = Pattern::new(
    "service-boundary",
    "service-boundaries.md",
    "Check DTO conversion, schema evolution, idempotency, and adapter-level resilience.",
);

const ERROR_CHAIN_LOG: Pattern = Pattern::new(
    "error-chain-log",
    "logging-metrics.md",
    "Check error chain preservation, single authoritative log line, and bounded metric labels.",
);

const PROPERTY_TEST: Pattern = Pattern::new(
    "property-test",
    "property-based-tests.md",
    "Check constructor-based generators, explicit properties, and no live I/O in properties.",
);

const PII_TERMS: &[&str] = &[
    "email",
    "phone",
    "address",
    "password",
    "passwd",
    "token",
    "secret",
    "api_key",
    "ssn",
    "patient",
    "diagnosis",
    "ip_address",
    "location",
];

const PERSISTENCE_TERMS: &[&str] = &[
    "transaction",
    "commit",
    "rollback",
    "outbox",
    "publish",
    "event",
    "save",
    "insert",
    "update",
    "delete",
    "idempot",
    "retry",
    "version",
    "lock",
];

const STREAM_TERMS: &[&str] = &[
    "Stream",
    "StreamExt",
    "stream",
    "async_stream",
    "projection",
    "subscribe",
    "checkpoint",
    "outbox",
];

const SERVICE_SEGMENTS: &[&str] = &[
    "tonic",
    "prost",
    "CircuitBreaker",
    "circuit_breaker",
    "idempotency",
    "schema_version",
    "grpc",
    "protobuf",
];

fn matches_term(ident: &str, terms: &[&str]) -> bool {
    let lower = ident.to_ascii_lowercase();
    terms.iter().any(|term| {
        let term_lower = term.to_ascii_lowercase();
        lower == term_lower || lower.contains(&term_lower)
    })
}

fn path_matches(path: &syn::Path, segments: &[&str]) -> bool {
    path.segments.iter().any(|segment| {
        let ident = segment.ident.to_string();
        segments
            .iter()
            .any(|term| ident == *term || ident.eq_ignore_ascii_case(term))
    })
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn macro_name(mac: &Macro) -> Option<String> {
    mac.path
        .get_ident()
        .map(|ident| ident.to_string())
        .or_else(|| {
            mac.path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
        })
}

fn doc_text(attrs: &[Attribute]) -> String {
    let mut docs = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let Meta::NameValue(meta) = &attr.meta else {
            continue;
        };
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = &meta.value
        {
            docs.push(lit.value());
        }
    }
    docs.join("\n")
}

fn has_outer_doc(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}

fn item_signature(source: &str, item: &Item) -> (usize, String) {
    let span = match item {
        Item::Fn(item) => item.sig.ident.span(),
        Item::Struct(item) => item.ident.span(),
        Item::Enum(item) => item.ident.span(),
        Item::Trait(item) => item.ident.span(),
        Item::Type(item) => item.ident.span(),
        _ => return (1, String::new()),
    };
    let line = line_of(span);
    let snippet = snippet_at(source, line);
    (line, truncate(snippet, 160))
}

impl<'ast> Visit<'ast> for ProbeVisitor<'ast> {
    fn visit_file(&mut self, file: &'ast syn::File) {
        for attr in &file.attrs {
            self.visit_attribute(attr);
        }
        visit::visit_file(self, file);
    }

    fn visit_item(&mut self, item: &'ast Item) {
        if let Item::Macro(ItemMacro { mac, .. }) = item {
            if let Some(name) = macro_name(mac) {
                if name == "nutype" || name == "macro_rules" {
                    let line = line_of(mac.span());
                    self.push_lead(DOMAIN_MACRO, line, snippet_at(self.source, line));
                }
            }
        }

        if is_public_item(item) {
            let (line, signature) = item_signature(self.source, item);
            if !signature.is_empty() {
                let doc = doc_text(item_attrs(item));
                self.public_items.push(PublicItem {
                    path: self.path.clone(),
                    line,
                    signature,
                    has_doc: has_outer_doc(item_attrs(item)),
                    has_errors: doc.contains("# Errors"),
                    has_panics: doc.contains("# Panics"),
                    has_safety: doc.contains("# Safety"),
                });
            }
        }

        if let Item::Fn(ItemFn { sig, .. }) = item {
            if sig.unsafety.is_some() {
                let line = line_of(sig.ident.span());
                self.push_lead(UNSAFE, line, snippet_at(self.source, line));
            }
        }

        visit::visit_item(self, item);
    }

    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        let path = attr.path().get_ident().map(|ident| ident.to_string());
        match path.as_deref() {
            Some("allow") => {
                let line = line_of(attr.span());
                self.push_lead(LINT_SUPPRESSION, line, snippet_at(self.source, line));
            }
            Some("derive") => {
                if let Meta::List(list) = &attr.meta {
                    self.visit_meta_list_tokens(
                        BOUNDARY_DERIVE,
                        attr.span(),
                        &list.tokens.to_string(),
                    );
                }
            }
            Some("source") | Some("from") => {
                let line = line_of(attr.span());
                self.push_lead(ERROR_CHAIN_LOG, line, snippet_at(self.source, line));
            }
            Some("proc_macro") | Some("proc_macro_attribute") | Some("proc_macro_derive") => {
                let line = line_of(attr.span());
                self.push_lead(DOMAIN_MACRO, line, snippet_at(self.source, line));
            }
            _ => {}
        }

        if let Meta::List(list) = &attr.meta {
            let tokens = list.tokens.to_string();
            if tokens.contains("DomainEvent") || tokens.contains("Newtype") {
                self.push_lead_span(DOMAIN_MACRO, attr.span(), tokens);
            }
        }

        visit::visit_attribute(self, attr);
    }

    fn visit_expr_unsafe(&mut self, expr: &'ast ExprUnsafe) {
        let line = line_of(expr.unsafe_token.span);
        self.push_lead(UNSAFE, line, snippet_at(self.source, line));
        visit::visit_expr_unsafe(self, expr);
    }

    fn visit_type_ptr(&mut self, ty: &'ast TypePtr) {
        let line = line_of(ty.star_token.span);
        self.push_lead(UNSAFE, line, snippet_at(self.source, line));
        visit::visit_type_ptr(self, ty);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if method == "unwrap" || method == "expect" {
            let line = line_of(call.method.span());
            self.push_lead(PANIC_PATH, line, snippet_at(self.source, line));
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, mac: &'ast ExprMacro) {
        if let Some(name) = macro_name(&mac.mac) {
            match name.as_str() {
                "panic" | "todo" | "unimplemented" => {
                    let line = line_of(mac.mac.span());
                    self.push_lead(PANIC_PATH, line, snippet_at(self.source, line));
                }
                "proptest" | "prop_assume" | "prop_assert" => {
                    let line = line_of(mac.mac.span());
                    self.push_lead(PROPERTY_TEST, line, snippet_at(self.source, line));
                }
                "stream" | "async_stream" => {
                    let line = line_of(mac.mac.span());
                    self.push_lead(STREAM_PROJECTION, line, snippet_at(self.source, line));
                }
                _ => {}
            }
        }
        visit::visit_expr_macro(self, mac);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path_expr) = &*call.func {
            let path = path_string(&path_expr.path);
            if path == "spawn" || path.ends_with("::spawn") || path.contains("tokio::spawn") {
                let line = line_of(path_expr.path.segments[0].ident.span());
                self.push_lead(ASYNC_OPERATIONAL, line, snippet_at(self.source, line));
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_await(&mut self, expr: &'ast ExprAwait) {
        let line = line_of(expr.await_token.span);
        self.push_lead(ASYNC_OPERATIONAL, line, snippet_at(self.source, line));
        visit::visit_expr_await(self, expr);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path_matches(path, SERVICE_SEGMENTS) {
            let line = line_of(path.segments[0].ident.span());
            self.push_lead(SERVICE_BOUNDARY, line, snippet_at(self.source, line));
        }
        if path_matches(path, &["Mutex", "RwLock"]) {
            let line = line_of(path.segments[0].ident.span());
            self.push_lead(ASYNC_OPERATIONAL, line, snippet_at(self.source, line));
        }
        if path_matches(
            path,
            &[
                "MaybeUninit",
                "transmute",
                "from_raw",
                "as_ptr",
                "as_mut_ptr",
            ],
        ) {
            let line = line_of(path.segments[0].ident.span());
            self.push_lead(UNSAFE, line, snippet_at(self.source, line));
        }
        if path_matches(
            path,
            &["proptest", "quickcheck", "ProptestConfig", "Strategy"],
        ) {
            let line = line_of(path.segments[0].ident.span());
            self.push_lead(PROPERTY_TEST, line, snippet_at(self.source, line));
        }
        if path_matches(path, STREAM_TERMS) {
            let line = line_of(path.segments[0].ident.span());
            self.push_lead(STREAM_PROJECTION, line, snippet_at(self.source, line));
        }
        visit::visit_path(self, path);
    }

    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        let name = ident.to_string();
        if matches_term(&name, PII_TERMS) {
            self.push_ident_lead(PII_SECRET, ident);
        }
        if matches_term(&name, PERSISTENCE_TERMS) {
            self.push_ident_lead(PERSISTENCE_EVENT, ident);
        }
        visit::visit_ident(self, ident);
    }

    fn visit_type_path(&mut self, ty: &'ast TypePath) {
        if path_matches(&ty.path, &["Mutex", "RwLock"]) {
            let line = line_of(ty.path.segments[0].ident.span());
            self.push_lead(ASYNC_OPERATIONAL, line, snippet_at(self.source, line));
        }
        visit::visit_type_path(self, ty);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        let tokens = mac.tokens.to_string();
        self.visit_error_log_tokens(mac.span(), &tokens);
        if let Some(name) = macro_name(mac) {
            if name == "macro_rules" {
                let line = line_of(mac.span());
                self.push_lead(DOMAIN_MACRO, line, snippet_at(self.source, line));
            }
        }
        visit::visit_macro(self, mac);
    }
}

fn is_public_item(item: &Item) -> bool {
    let vis = match item {
        Item::Fn(i) => &i.vis,
        Item::Struct(i) => &i.vis,
        Item::Enum(i) => &i.vis,
        Item::Trait(i) => &i.vis,
        Item::Type(i) => &i.vis,
        _ => return false,
    };
    matches!(vis, Visibility::Public(_))
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Fn(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        _ => &[],
    }
}

pub enum CollectOutcome {
    Skipped,
    Collected {
        leads: Vec<Lead>,
        public_items: Vec<PublicItem>,
    },
    ParseError(syn::Error),
}

pub fn collect_file(path: &Path, rel: String) -> std::io::Result<CollectOutcome> {
    let source = std::fs::read_to_string(path)?;
    if crate::scan::is_generated(path, &source) {
        return Ok(CollectOutcome::Skipped);
    }

    let syntax = match syn::parse_file(&source) {
        Ok(syntax) => syntax,
        Err(error) => return Ok(CollectOutcome::ParseError(error)),
    };
    let mut visitor = ProbeVisitor::new(rel, &source);
    visitor.visit_file(&syntax);
    Ok(CollectOutcome::Collected {
        leads: visitor.leads,
        public_items: visitor.public_items,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn signature_helpers() {
        assert!("pub fn f() -> Result<(), E>".contains("Result"));
        assert!("pub unsafe fn f()".contains("unsafe "));
    }
}
