#!/usr/bin/env python3
"""Collect review leads for kamae-rs without external dependencies."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


ROOT = Path.cwd()
SKIP_DIRS = {".git", ".dagayn", "target", "node_modules", ".venv", "venv"}


@dataclass(frozen=True)
class Pattern:
    category: str
    checklist: str
    regex: re.Pattern[str]
    note: str


@dataclass
class Lead:
    category: str
    checklist: str
    path: str
    line: int
    snippet: str
    note: str


@dataclass
class PublicItem:
    path: str
    line: int
    signature: str
    has_doc: bool
    has_errors: bool
    has_panics: bool
    has_safety: bool


PATTERNS = [
    Pattern(
        "unsafe",
        "unsafe-boundaries.md",
        re.compile(
            r"\bunsafe\b|\bMaybeUninit\b|\btransmute\b|\bfrom_raw\b|"
            r"\bas_ptr\b|\bas_mut_ptr\b|\*const\b|\*mut\b"
        ),
        "Inspect unsafe soundness, containment behind a safe API, and safety comments.",
    ),
    Pattern(
        "lint-suppression",
        "fmt-lint.md",
        re.compile(r"#\s*!?\s*\[\s*allow\s*\(|allow\s*\(\s*warnings\s*\)|clippy::all"),
        "Check whether lint suppression is narrow and justified.",
    ),
    Pattern(
        "panic-path",
        "error-handling.md",
        re.compile(r"\bunwrap\s*\(|\bexpect\s*\(|panic!\s*\(|todo!\s*\(|unimplemented!\s*\("),
        "Confirm this is test/startup/proven-invariant code or replace with typed errors.",
    ),
    Pattern(
        "boundary-derive",
        "boundary.md",
        re.compile(r"Deserialize|Serialize|FromRow|serde\s*\(\s*default|deny_unknown_fields"),
        "Check DTO/domain separation, validation, defaults, and serialization intent.",
    ),
    Pattern(
        "pii-secret",
        "pii-protection.md",
        re.compile(
            r"\b(email|phone|address|password|passwd|token|secret|api_key|ssn|"
            r"patient|diagnosis|ip_address|location)\b"
        ),
        "Check redaction, Debug/log exposure, and plaintext access.",
    ),
    Pattern(
        "persistence-event",
        "persistence-events.md",
        re.compile(
            r"\b(transaction|commit|rollback|outbox|publish|event|save|insert|update|delete|"
            r"idempot|retry|version|lock)\b"
        ),
        "Check atomicity, idempotency, DB constraints, and event versioning.",
    ),
    Pattern(
        "async-operational",
        "persistence-events.md",
        re.compile(r"\btokio::spawn\b|spawn\s*\(|await\b|Mutex|RwLock"),
        "Check lost task failures, transaction boundaries across await, and ignored Results.",
    ),
    Pattern(
        "stream-projection",
        "stream-continuous-queries.md",
        re.compile(
            r"\bStream\b|\bStreamExt\b|\bstream!\b|\basync_stream\b|"
            r"\bprojection\b|\bsubscribe\b|\bcheckpoint\b|\boutbox\b"
        ),
        "Check durable cursors, backpressure, projection idempotency, and CQRS boundaries.",
    ),
    Pattern(
        "domain-macro",
        "domain-macros.md",
        re.compile(
            r"proc_macro|macro_rules!\s|#\[derive\s*\(\s*DomainEvent|"
            r"#\[derive\s*\(\s*Newtype|nutype\s*\("
        ),
        "Check generated invariants, safe Debug, and schema/version metadata.",
    ),
    Pattern(
        "service-boundary",
        "service-boundaries.md",
        re.compile(
            r"\btonic::|\bprost::|\bCircuitBreaker\b|\bcircuit_breaker\b|"
            r"\bidempotency\b|\bschema_version\b|\bgrpc\b|\bprotobuf\b"
        ),
        "Check DTO conversion, schema evolution, idempotency, and adapter-level resilience.",
    ),
    Pattern(
        "error-chain-log",
        "logging-metrics.md",
        re.compile(r"#\[source\]|#\[from\]|error\s*=\s*%|error\.debug\s*="),
        "Check error chain preservation, single authoritative log line, and bounded metric labels.",
    ),
]

PUBLIC_ITEM_RE = re.compile(
    r"^\s*pub(?:\([^)]*\))?\s+(?:async\s+)?(?:unsafe\s+)?"
    r"(struct|enum|trait|fn|type)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


def is_generated(path: Path, text: str) -> bool:
    rel = path.as_posix().lower()
    head = "\n".join(text.splitlines()[:8]).lower()
    return (
        "generated" in rel
        or "bindings" in rel
        or "@generated" in head
        or "automatically generated" in head
        or "do not edit" in head
    )


def iter_rust_files(paths: Iterable[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if not path.exists():
            continue
        if path.is_file() and path.suffix == ".rs":
            files.append(path)
            continue
        if path.is_dir():
            for child in path.rglob("*.rs"):
                if any(part in SKIP_DIRS for part in child.parts):
                    continue
                files.append(child)
    return sorted(set(files))


def rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def has_doc_before(lines: list[str], index: int) -> bool:
    cursor = index - 1
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if not stripped:
            cursor -= 1
            continue
        if stripped.startswith("#["):
            cursor -= 1
            continue
        return stripped.startswith("///") or stripped.startswith("#[doc")
    return False


def doc_block_before(lines: list[str], index: int) -> str:
    docs: list[str] = []
    cursor = index - 1
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if not stripped or stripped.startswith("#["):
            cursor -= 1
            continue
        if stripped.startswith("///"):
            docs.append(stripped[3:].strip())
            cursor -= 1
            continue
        if stripped.startswith("#[doc"):
            docs.append(stripped)
            cursor -= 1
            continue
        break
    docs.reverse()
    return "\n".join(docs)


def collect_file(path: Path) -> tuple[list[Lead], list[PublicItem]]:
    text = path.read_text(encoding="utf-8", errors="replace")
    if is_generated(path, text):
        return [], []

    leads: list[Lead] = []
    public_items: list[PublicItem] = []
    lines = text.splitlines()

    for line_no, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("//"):
            continue
        for pattern in PATTERNS:
            if pattern.regex.search(stripped):
                leads.append(
                    Lead(
                        category=pattern.category,
                        checklist=pattern.checklist,
                        path=rel(path),
                        line=line_no,
                        snippet=stripped[:160],
                        note=pattern.note,
                    )
                )

    for index, line in enumerate(lines):
        match = PUBLIC_ITEM_RE.match(line)
        if not match:
            continue
        doc = doc_block_before(lines, index)
        signature = line.strip()
        has_doc = has_doc_before(lines, index)
        public_items.append(
            PublicItem(
                path=rel(path),
                line=index + 1,
                signature=signature[:160],
                has_doc=has_doc,
                has_errors="# Errors" in doc,
                has_panics="# Panics" in doc,
                has_safety="# Safety" in doc,
            )
        )
    return leads, public_items


def cargo_commands(paths: list[Path]) -> list[str]:
    has_cargo = any((path / "Cargo.toml").is_file() for path in [ROOT, *ROOT.parents])
    if not has_cargo and not Path("Cargo.toml").is_file():
        return []
    commands = ["cargo fmt --check", "cargo clippy --all-targets --all-features", "cargo test"]
    if any(item.has_doc for path in paths for item in collect_file(path)[1]):
        commands.append("cargo doc --no-deps")
    return commands


def summarize(leads: list[Lead], public_items: list[PublicItem]) -> dict[str, object]:
    by_category: dict[str, int] = {}
    by_checklist: dict[str, int] = {}
    for lead in leads:
        by_category[lead.category] = by_category.get(lead.category, 0) + 1
        by_checklist[lead.checklist] = by_checklist.get(lead.checklist, 0) + 1

    missing_docs = [item for item in public_items if not item.has_doc]
    unsafe_without_safety = [
        item for item in public_items if "unsafe" in item.signature and not item.has_safety
    ]
    result_without_errors = [
        item
        for item in public_items
        if "Result" in item.signature and item.has_doc and not item.has_errors
    ]

    return {
        "leads_by_category": by_category,
        "leads_by_checklist": by_checklist,
        "public_items": len(public_items),
        "public_items_missing_docs": len(missing_docs),
        "unsafe_public_items_missing_safety": len(unsafe_without_safety),
        "documented_result_items_missing_errors": len(result_without_errors),
    }


def render_text(
    files: list[Path],
    leads: list[Lead],
    public_items: list[PublicItem],
    commands: list[str],
    limit: int,
) -> str:
    summary = summarize(leads, public_items)
    output: list[str] = [
        "# kamae-rs Review Probe",
        "",
        "These are review leads, not findings. Confirm reachability, invariants, and project conventions before reporting.",
        "",
        f"Scanned Rust files: {len(files)}",
    ]

    if commands:
        output.extend(["", "## Suggested Commands"])
        output.extend(f"- `{command}`" for command in commands)

    output.extend(["", "## Summary"])
    for key, value in summary.items():
        output.append(f"- {key}: {value}")

    output.extend(["", "## Leads"])
    if not leads:
        output.append("- No pattern leads found.")
    else:
        for lead in leads[:limit]:
            output.append(
                f"- `{lead.path}:{lead.line}` [{lead.category} -> {lead.checklist}] "
                f"{lead.snippet}\n  {lead.note}"
            )
        if len(leads) > limit:
            output.append(f"- Truncated {len(leads) - limit} additional lead(s). Use `--limit` to show more.")

    missing_docs = [item for item in public_items if not item.has_doc]
    result_without_errors = [
        item
        for item in public_items
        if "Result" in item.signature and item.has_doc and not item.has_errors
    ]
    unsafe_without_safety = [
        item for item in public_items if "unsafe" in item.signature and not item.has_safety
    ]

    output.extend(["", "## Rustdoc Leads"])
    for title, items, note in [
        ("Public items missing rustdoc", missing_docs, "Review whether this public API needs a domain contract."),
        ("Documented Result items missing # Errors", result_without_errors, "Callers may need error semantics."),
        ("Unsafe public items missing # Safety", unsafe_without_safety, "Unsafe caller obligations must be explicit."),
    ]:
        output.append(f"### {title}")
        if not items:
            output.append("- None.")
            continue
        for item in items[:limit]:
            output.append(f"- `{item.path}:{item.line}` {item.signature}\n  {note}")
        if len(items) > limit:
            output.append(f"- Truncated {len(items) - limit} additional item(s).")

    return "\n".join(output)


def run_json(
    files: list[Path],
    leads: list[Lead],
    public_items: list[PublicItem],
    commands: list[str],
) -> str:
    data = {
        "scanned_files": [rel(path) for path in files],
        "summary": summarize(leads, public_items),
        "suggested_commands": commands,
        "leads": [lead.__dict__ for lead in leads],
        "public_items": [item.__dict__ for item in public_items],
    }
    return json.dumps(data, indent=2, sort_keys=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Collect kamae-rs review leads from Rust files.")
    parser.add_argument("paths", nargs="*", default=["."], help="Files or directories to scan.")
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown text.")
    parser.add_argument("--limit", type=int, default=80, help="Maximum leads per text section.")
    args = parser.parse_args()

    paths = [Path(path) for path in args.paths]
    files = iter_rust_files(paths)
    leads: list[Lead] = []
    public_items: list[PublicItem] = []
    for path in files:
        file_leads, file_items = collect_file(path)
        leads.extend(file_leads)
        public_items.extend(file_items)

    commands = cargo_commands(files)
    if args.json:
        print(run_json(files, leads, public_items, commands))
    else:
        print(render_text(files, leads, public_items, commands, max(args.limit, 1)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
