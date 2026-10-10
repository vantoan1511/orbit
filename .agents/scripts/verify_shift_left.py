#!/usr/bin/env python3
"""
verify_shift_left.py — Orbit Shift-Left Pre-Flight Verification Tool

Fast, zero-dependency (<0.5s) static validation script that enforces:
1. Git branch discipline (never main/master)
2. Architecture boundaries (no @neutralinojs/lib in components/composables)
3. PrimeVue component enforcement (no raw <button>, <input>, <select> in templates)
4. Tooltip directive enforcement (no native title="..." in templates)
5. Design token compliance (no hardcoded hex colors in styles)
6. Rust backend safety (no unwrap/expect/panic in production code)
"""

import sys
import os
import re
import subprocess
import argparse
from pathlib import Path

# ANSI colors
COLOR_RESET = "\033[0m"
COLOR_RED = "\033[31m"
COLOR_GREEN = "\033[32m"
COLOR_YELLOW = "\033[33m"
COLOR_BLUE = "\033[34m"
COLOR_CYAN = "\033[36m"
COLOR_BOLD = "\033[1m"

# Ensure UTF-8 output on Windows consoles
if sys.stdout.encoding and sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

# Safe markers for cross-platform terminals
MARK_PASS = "[PASS]"
MARK_FAIL = "[FAIL]"
MARK_WARN = "[WARN]"

def print_header(title: str):
    print(f"\n{COLOR_BOLD}{COLOR_CYAN}==> {title}{COLOR_RESET}")

def print_success(msg: str):
    print(f"{COLOR_GREEN}{MARK_PASS} {msg}{COLOR_RESET}")

def print_warning(msg: str):
    print(f"{COLOR_YELLOW}{MARK_WARN} {msg}{COLOR_RESET}")

def print_error(msg: str):
    print(f"{COLOR_RED}{MARK_FAIL} {msg}{COLOR_RESET}")

class Violation:
    def __init__(self, file: str, line: int, code: str, rule: str, suggestion: str):
        self.file = file
        self.line = line
        self.code = code.strip()
        self.rule = rule
        self.suggestion = suggestion

    def __str__(self):
        return (
            f"  {COLOR_RED}[Line {self.line}]{COLOR_RESET} {COLOR_BOLD}{self.rule}{COLOR_RESET}\n"
            f"    Snippet:    `{self.code}`\n"
            f"    Suggestion: {COLOR_YELLOW}{self.suggestion}{COLOR_RESET}"
        )

def get_repo_root() -> Path:
    try:
        res = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=True
        )
        return Path(res.stdout.strip())
    except Exception:
        # Fallback to current directory or parent
        return Path(__file__).resolve().parent.parent.parent

def get_current_branch(root: Path) -> str:
    try:
        res = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=root,
            capture_output=True,
            text=True,
            check=True
        )
        return res.stdout.strip()
    except Exception:
        return ""

def get_changed_files(root: Path, staged_only: bool = False) -> list[Path]:
    files = set()
    try:
        if staged_only:
            cmd = ["git", "diff", "--name-only", "--cached", "--diff-filter=ACMRT"]
            res = subprocess.run(cmd, cwd=root, capture_output=True, text=True, check=True)
            for f in res.stdout.splitlines():
                if f.strip():
                    files.add(root / f.strip())
        else:
            # Check uncommitted changes (both staged and unstaged) + untracked files
            cmd1 = ["git", "diff", "--name-only", "HEAD", "--diff-filter=ACMRT"]
            res1 = subprocess.run(cmd1, cwd=root, capture_output=True, text=True)
            if res1.returncode == 0:
                for f in res1.stdout.splitlines():
                    if f.strip():
                        files.add(root / f.strip())
            else:
                # Fallback if HEAD does not exist (new repo)
                cmd_staged = ["git", "diff", "--name-only", "--cached"]
                res_staged = subprocess.run(cmd_staged, cwd=root, capture_output=True, text=True)
                for f in res_staged.stdout.splitlines():
                    if f.strip():
                        files.add(root / f.strip())

            # Untracked files
            cmd2 = ["git", "ls-files", "--others", "--exclude-standard"]
            res2 = subprocess.run(cmd2, cwd=root, capture_output=True, text=True)
            if res2.returncode == 0:
                for f in res2.stdout.splitlines():
                    if f.strip():
                        files.add(root / f.strip())
    except Exception as e:
        print_warning(f"Could not determine git changed files: {e}")
        return []

    return [f for f in files if f.exists() and f.is_file()]

def check_branch_safety(branch: str) -> list[str]:
    violations = []
    if branch in ("main", "master"):
        violations.append(
            f"Current branch is '{branch}'. Never commit or implement directly on main/master! "
            f"Run `git checkout -b feat/<name>` or `fix/<name>` first."
        )
    return violations

def check_neutralino_imports(file_path: Path, content: str) -> list[Violation]:
    """Rule: No @neutralinojs/lib in components, views, or composables."""
    violations = []
    norm_path = file_path.as_posix()
    
    # Allowed areas for Neutralino imports: src/services/, src/main.ts, tests
    is_ui_or_composable = any(
        target in norm_path
        for target in ("src/components/", "src/views/", "src/composables/")
    )
    if not is_ui_or_composable:
        return violations

    for idx, line in enumerate(content.splitlines(), start=1):
        if re.search(r"['\"]@neutralinojs/lib['\"]", line):
            if "shift-left-ignore" in line:
                continue
            violations.append(
                Violation(
                    file=norm_path,
                    line=idx,
                    code=line,
                    rule="Direct `@neutralinojs/lib` import in UI/composable",
                    suggestion="Import capabilities via `@/services/nativeService` or specialized domain services instead."
                )
            )
    return violations

def check_primevue_elements(file_path: Path, content: str) -> list[Violation]:
    """Rule: No raw <button>, <input>, <select> in Vue templates."""
    violations = []
    norm_path = file_path.as_posix()
    if not norm_path.endswith(".vue"):
        return violations

    template_match = re.search(r"<template\b[^>]*>(.*?)</template>", content, re.DOTALL)
    if not template_match:
        return violations

    template_content = template_match.group(1)
    template_start_pos = template_match.start(1)

    for m in re.finditer(r"<([a-zA-Z0-9_-]+)(\s*[^>]*?)>", template_content, re.DOTALL):
        tag_name = m.group(1)
        tag_attrs = m.group(2)
        tag_full = m.group(0)

        if "shift-left-ignore" in tag_attrs:
            continue

        line_num = content[: template_start_pos + m.start()].count("\n") + 1
        first_line = tag_full.splitlines()[0]

        if tag_name == "button":
            violations.append(
                Violation(
                    file=norm_path,
                    line=line_num,
                    code=first_line,
                    rule="Raw `<button>` HTML tag in Vue template",
                    suggestion="Use PrimeVue `<Button>` component instead."
                )
            )
        elif tag_name == "input":
            violations.append(
                Violation(
                    file=norm_path,
                    line=line_num,
                    code=first_line,
                    rule="Raw `<input>` HTML tag in Vue template",
                    suggestion="Use PrimeVue `<InputText>`, `<InputNumber>`, or `<Checkbox>` instead."
                )
            )
        elif tag_name == "select":
            violations.append(
                Violation(
                    file=norm_path,
                    line=line_num,
                    code=first_line,
                    rule="Raw `<select>` HTML tag in Vue template",
                    suggestion="Use PrimeVue `<Select>` component instead."
                )
            )

    return violations

def check_tooltip_directive(file_path: Path, content: str) -> list[Violation]:
    """Rule: No native title="..." attributes in Vue templates; use v-tooltip."""
    violations = []
    norm_path = file_path.as_posix()
    if not norm_path.endswith(".vue"):
        return violations

    template_match = re.search(r"<template\b[^>]*>(.*?)</template>", content, re.DOTALL)
    if not template_match:
        return violations

    template_content = template_match.group(1)
    template_start_pos = template_match.start(1)

    # Components where 'title' is a legitimate component prop, not an HTML tooltip
    PROP_TITLE_COMPONENTS = {
        "KeyValueEditor",
        "KeyValueBadgeList",
        "BaseResourceDrawer",
        "Card",
        "Panel",
        "Dialog",
        "Drawer",
        "TabPanel",
        "ViewLayout",
        "EmptyState",
        "SectionHeader",
        "DetailsCard",
    }

    for m in re.finditer(r"<([a-zA-Z0-9_-]+)(\s*[^>]*?)>", template_content, re.DOTALL):
        tag_name = m.group(1)
        tag_attrs = m.group(2)
        tag_full = m.group(0)

        if tag_name.lower() == "title" or tag_name in PROP_TITLE_COMPONENTS:
            continue

        if "shift-left-ignore" in tag_attrs:
            continue

        # Check for title="..." attribute on the tag (exclude dynamic :title on custom components)
        title_attr = re.search(r"(?<!:)\btitle\s*=\s*(['\"][^'\"]*?['\"])", tag_attrs)
        if title_attr:
            # If it's a custom PascalCase component (other than PrimeVue components), title is often a prop
            PRIMEVUE_COMPONENTS = {
                "Button", "InputText", "InputNumber", "Select", "Textarea",
                "ToggleSwitch", "Checkbox", "RadioButton", "Tag", "Badge",
                "Tab", "Column", "DataTable", "Message"
            }
            if tag_name[0].isupper() and tag_name not in PRIMEVUE_COMPONENTS:
                # Custom component prop
                continue

            # Find the specific line of the title attribute
            attr_pos_in_tag = title_attr.start()
            line_num = content[: template_start_pos + m.start() + attr_pos_in_tag].count("\n") + 1
            snippet = title_attr.group(0)

            violations.append(
                Violation(
                    file=norm_path,
                    line=line_num,
                    code=snippet,
                    rule=f"Native HTML `title` tooltip on `<{tag_name}>` in Vue template",
                    suggestion="Use PrimeVue `v-tooltip=\"'...'\"` directive instead of native title attribute."
                )
            )

    return violations

def check_design_tokens(file_path: Path, content: str) -> list[Violation]:
    """Rule: No hardcoded hex colors in CSS/styles (excluding base.css token definitions)."""
    violations = []
    norm_path = file_path.as_posix()

    # Skip base.css because it defines the canonical tokens
    if "src/assets/base.css" in norm_path or "node_modules" in norm_path:
        return violations

    blocks_to_check = []
    if norm_path.endswith((".css", ".scss")):
        blocks_to_check.append((1, content))
    elif norm_path.endswith(".vue"):
        for m in re.finditer(r"<style\b[^>]*>(.*?)</style>", content, re.DOTALL):
            start_line = content[:m.start(1)].count("\n") + 1
            blocks_to_check.append((start_line, m.group(1)))

    for start_line, block in blocks_to_check:
        lines = block.splitlines()
        in_comment = False
        for offset, line in enumerate(lines):
            line_num = start_line + offset
            stripped = line.strip()
            if "/*" in stripped and "*/" not in stripped:
                in_comment = True
                continue
            if in_comment:
                if "*/" in stripped:
                    in_comment = False
                continue
            if "shift-left-ignore" in line or line.strip().startswith("//"):
                continue

            # Look for hex colors #xxx or #xxxxxx
            hex_match = re.search(r"#[0-9a-fA-F]{3,8}\b", line)
            if hex_match:
                # Ignore base64 encoded strings or urls
                if "data:image" in line or "url(" in line:
                    continue
                violations.append(
                    Violation(
                        file=norm_path,
                        line=line_num,
                        code=line,
                        rule=f"Hardcoded hex color `{hex_match.group(0)}` in stylesheet",
                        suggestion="Use Orbit CSS tokens (var(--accent), var(--border), var(--bg-card), etc.) from base.css."
                    )
                )

    return violations

def check_rust_safety(file_path: Path, content: str) -> list[Violation]:
    """Rule: No unwrap(), expect(), or panic! in production Rust backend code."""
    violations = []
    norm_path = file_path.as_posix()
    if not norm_path.endswith(".rs"):
        return violations

    # Skip dedicated tests/ directory or build scripts
    if "/tests/" in norm_path or norm_path.startswith("tests/") or norm_path.endswith("build.rs"):
        return violations

    lines = content.splitlines()
    in_test_module = False
    test_brace_depth = 0

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Track #[cfg(test)] module boundaries
        if "#[cfg(test)]" in line:
            in_test_module = True
            test_brace_depth = 0
            continue

        if in_test_module:
            test_brace_depth += line.count("{") - line.count("}")
            if test_brace_depth <= 0 and "}" in line:
                in_test_module = False
            continue

        if "shift-left-ignore" in line or stripped.startswith("//"):
            continue

        # Look for .unwrap(), .expect(...), panic!(...)
        if re.search(r"\.unwrap\(\)", line):
            violations.append(
                Violation(
                    file=norm_path,
                    line=idx,
                    code=line,
                    rule="Rust `.unwrap()` call in production engine code",
                    suggestion="Handle errors explicitly with `?` or `match` / `if let` returning Result<T, E>."
                )
            )
        elif re.search(r"\.expect\(", line):
            violations.append(
                Violation(
                    file=norm_path,
                    line=idx,
                    code=line,
                    rule="Rust `.expect(...)` call in production engine code",
                    suggestion="Handle recoverable errors explicitly or log diagnostic failure via Result<T, E>."
                )
            )
        elif re.search(r"\bpanic!\(", line):
            violations.append(
                Violation(
                    file=norm_path,
                    line=idx,
                    code=line,
                    rule="Rust `panic!(...)` macro in production engine code",
                    suggestion="Return structured errors across IPC boundary instead of crashing backend engine."
                )
            )

    return violations

def run_command_check(name: str, cmd: list[str], cwd: Path) -> bool:
    print_header(f"Running {name}: {' '.join(cmd)}")
    try:
        use_shell = sys.platform == "win32"
        res = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, shell=use_shell)
        if res.returncode == 0:
            print_success(f"{name} passed.")
            return True
        else:
            print_error(f"{name} failed with exit code {res.returncode}:")
            if res.stdout.strip():
                print(res.stdout)
            if res.stderr.strip():
                print(res.stderr)
            return False
    except Exception as e:
        print_error(f"Failed to execute {name}: {e}")
        return False

def scan_files(files: list[Path]) -> dict[str, list[Violation]]:
    results = {}
    for f in files:
        try:
            content = f.read_text(encoding="utf-8")
        except Exception:
            continue

        file_violations = []
        file_violations.extend(check_neutralino_imports(f, content))
        file_violations.extend(check_primevue_elements(f, content))
        file_violations.extend(check_tooltip_directive(f, content))
        file_violations.extend(check_design_tokens(f, content))
        file_violations.extend(check_rust_safety(f, content))

        if file_violations:
            results[f.as_posix()] = file_violations

    return results

def main():
    parser = argparse.ArgumentParser(description="Orbit Shift-Left Verification Scanner")
    parser.add_argument("--changed", action="store_true", default=True, help="Scan git changed and untracked files (default)")
    parser.add_argument("--staged", action="store_true", help="Scan git staged files only")
    parser.add_argument("--all", action="store_true", help="Scan all source files in repository")
    parser.add_argument("--files", nargs="*", help="Scan explicit list of files")
    parser.add_argument("--type-check", action="store_true", help="Run vue-tsc type checking")
    parser.add_argument("--lint", action="store_true", help="Run eslint")
    parser.add_argument("--test", action="store_true", help="Run frontend unit tests")
    parser.add_argument("--cargo-test", action="store_true", help="Run rust cargo test")
    parser.add_argument("--all-checks", action="store_true", help="Run pattern scanner + lint + type-check + tests")

    args = parser.parse_args()
    root = get_repo_root()

    print_header("Orbit Shift-Left Pre-Flight Check")

    # 1. Branch Safety Check
    branch = get_current_branch(root)
    print(f"Working Directory: {root}")
    print(f"Current Branch:    {COLOR_BOLD}{branch or 'Unknown'}{COLOR_RESET}")

    branch_violations = check_branch_safety(branch)
    if branch_violations:
        for bv in branch_violations:
            print_error(bv)
        print_error("Shift-Left Pre-Flight failed: Branch safety violation.")
        sys.exit(1)
    else:
        print_success("Branch safety verified (not on main/master).")

    # 2. Collect target files
    target_files = []
    if args.files:
        target_files = [Path(f).resolve() for f in args.files if Path(f).exists()]
    elif args.all:
        for ext in ("*.vue", "*.ts", "*.js", "*.css", "*.rs"):
            target_files.extend(root.glob(f"src/**/{ext}"))
            target_files.extend(root.glob(f"core/engine/src/**/{ext}"))
    elif args.staged:
        target_files = get_changed_files(root, staged_only=True)
    else: # changed
        target_files = get_changed_files(root, staged_only=False)

    # Filter out ignored paths
    filtered_files = [
        f for f in target_files
        if "node_modules" not in f.parts and "target" not in f.parts and ".git" not in f.parts
    ]

    print(f"Files to inspect:  {len(filtered_files)}")

    # 3. Scan files for pattern violations
    violations_by_file = scan_files(filtered_files)

    total_violations = sum(len(v) for v in violations_by_file.values())

    if total_violations > 0:
        print_header("Shift-Left Pattern Violations Found")
        for file_path, violations in violations_by_file.items():
            rel_path = os.path.relpath(file_path, root)
            print(f"\n{COLOR_BOLD}File: {rel_path}{COLOR_RESET} ({len(violations)} issues)")
            for v in violations:
                print(v)

        print_error(f"\nPre-Flight failed: {total_violations} shift-left violation(s) detected across {len(violations_by_file)} file(s).")
        print_warning("Fix the issues above before proceeding with implementation or committing.")
        sys.exit(1)
    else:
        print_success(f"Pattern checks clean: 0 violations across {len(filtered_files)} file(s).")

    # 4. Optional Tool Commands
    commands_passed = True
    if args.lint or args.all_checks:
        commands_passed = run_command_check("ESLint", ["npm", "run", "lint"], root) and commands_passed

    if args.type_check or args.all_checks:
        commands_passed = run_command_check("Type Check", ["npm", "run", "type-check"], root) and commands_passed

    if args.test or args.all_checks:
        commands_passed = run_command_check("Frontend Tests", ["npm", "test"], root) and commands_passed

    if args.cargo_test or args.all_checks:
        cargo_path = "src-tauri/Cargo.toml" if (root / "src-tauri/Cargo.toml").exists() else "core/Cargo.toml"
        commands_passed = run_command_check("Rust Tests", ["cargo", "test", "--manifest-path", cargo_path], root) and commands_passed

    if not commands_passed:
        print_error("\nOne or more validation commands failed.")
        sys.exit(1)

    print_header("Pre-Flight Complete")
    print_success("All shift-left gates passed cleanly!")
    sys.exit(0)

if __name__ == "__main__":
    main()
