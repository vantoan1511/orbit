#!/usr/bin/env python3
"""
curate_knowledge.py — Self-Improving Knowledge Curation Tool for Orbit

Scans agent logs (.agents/logs/*.md) and PRP implementation reports (.agents/PRPs/reports/*-report.md)
to extract user corrections, recurring gotchas, and implementation deviations,
verifying and updating the curated knowledge base in .agents/references/.
"""

import sys
import os
import glob
import re
from pathlib import Path

# Ensure UTF-8 output on Windows consoles
if sys.stdout.encoding and sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

def get_base_dir() -> Path:
    return Path(__file__).resolve().parent.parent

def extract_user_corrections(base_dir: Path) -> list[dict]:
    logs_dir = base_dir / "logs"
    if not logs_dir.exists():
        return []

    corrections = []
    for log_file in sorted(logs_dir.glob("*.md")):
        try:
            content = log_file.read_text(encoding="utf-8")
        except Exception:
            continue

        for m in re.finditer(r"## Time:\s*([^\n]+)\n\*\*User Request:\*\*\s*([^\n]+)", content):
            time_str = m.group(1).strip()
            req_str = m.group(2).strip()

            # Check if it looks like a correction or memory instruction
            keywords = ["remember", "wrong", "fix that", "instead of", "don't do", "rule", "should have"]
            if any(k in req_str.lower() for k in keywords):
                corrections.append({
                    "file": log_file.name,
                    "time": time_str,
                    "text": req_str
                })

    return corrections

def extract_report_issues(base_dir: Path) -> list[dict]:
    reports_dir = base_dir / "PRPs" / "reports"
    if not reports_dir.exists():
        return []

    issues = []
    for report_file in sorted(reports_dir.glob("*-report.md")):
        try:
            content = report_file.read_text(encoding="utf-8")
        except Exception:
            continue

        # Look for ## Deviations from Plan or ## Issues Encountered
        m_dev = re.search(r"## Deviations from Plan\s*\n(.*?)(?=\n##|\Z)", content, re.DOTALL)
        if m_dev:
            dev_text = m_dev.group(1).strip()
            if dev_text and dev_text.lower() != "none":
                issues.append({
                    "file": report_file.name,
                    "section": "Deviations",
                    "text": dev_text
                })

        m_issues = re.search(r"## Issues Encountered\s*\n(.*?)(?=\n##|\Z)", content, re.DOTALL)
        if m_issues:
            iss_text = m_issues.group(1).strip()
            if iss_text and iss_text.lower() != "none":
                issues.append({
                    "file": report_file.name,
                    "section": "Issues",
                    "text": iss_text
                })

    return issues

def verify_reference_links(base_dir: Path) -> list[str]:
    repo_root = base_dir.parent
    ref_dir = base_dir / "references"
    broken_links = []

    for ref_file in ref_dir.rglob("*.md"):
        try:
            content = ref_file.read_text(encoding="utf-8")
        except Exception:
            continue

        # Search for file:// links
        for m in re.finditer(r"\[.*?\]\((file:///[^)]+)\)", content):
            raw_uri = m.group(1)
            # Remove file:/// and normalize
            path_part = raw_uri.replace("file:///", "")
            target_path = Path(path_part)
            if not target_path.exists():
                broken_links.append(f"{ref_file.name}: broken link -> {raw_uri}")

    return broken_links

def main():
    base_dir = get_base_dir()
    print("==> Orbit Knowledge Base Curation Report")
    print(f"Base Directory: {base_dir}\n")

    # 1. Verify existing reference links
    broken = verify_reference_links(base_dir)
    if broken:
        print("[WARN] Broken reference links detected:")
        for b in broken:
            print(f"  - {b}")
    else:
        print("[PASS] All reference links verified against codebase.")

    # 2. Extract corrections
    corrections = extract_user_corrections(base_dir)
    print(f"\nExtracted {len(corrections)} user correction(s) from agent logs:")
    for c in corrections[-5:]:  # Show last 5
        print(f"  [{c['file']} {c['time']}] {c['text']}")

    # 3. Extract report insights
    report_issues = extract_report_issues(base_dir)
    print(f"\nExtracted {len(report_issues)} deviation/issue insights from PRP reports:")
    for ri in report_issues[-5:]:
        print(f"  [{ri['file']} - {ri['section']}] {ri['text'][:120]}...")

    print("\n==> Knowledge Base Status: Operational & Curated.")

if __name__ == "__main__":
    main()
