#!/usr/bin/env python3
"""
shift_left_hook.py — PreInvocation Lifecycle Hook for Orbit Shift-Left

Inspects the user's latest request in transcript.jsonl. If the user is requesting
implementation, planning, bug fixing, or refactoring, injects an ephemeral
system reminder to guide the agent toward shift-left practices (TDD, contract-first,
pre-flight scanning).
"""

import sys
import os
import json
import re

SHIFT_LEFT_KEYWORDS = [
    r"/implement\b",
    r"/plan\b",
    r"/prp-plan\b",
    r"\bimplement\b",
    r"\brefactor\b",
    r"\bfix\b",
    r"\badd\s+feature\b",
    r"\bcreate\b",
    r"\bbuild\b",
    r"\bupdate\b"
]

def main():
    try:
        # Hooks receive JSON payload on stdin
        if not sys.stdin.isatty():
            payload = json.load(sys.stdin)
        else:
            payload = {}

        transcript_path = payload.get("transcriptPath")
        if not transcript_path or not os.path.exists(transcript_path):
            print(json.dumps({}))
            return

        latest_user_request = ""
        with open(transcript_path, "r", encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    step = json.loads(line)
                    if step.get("type") == "USER_INPUT":
                        content = step.get("content", "")
                        match = re.search(r"<USER_REQUEST>\n?(.*?)\n?</USER_REQUEST>", content, re.DOTALL)
                        if match:
                            latest_user_request = match.group(1).strip()
                        else:
                            latest_user_request = content.strip()
                except Exception:
                    pass

        # Check if the request matches implementation/planning keywords
        is_dev_request = any(re.search(pat, latest_user_request, re.IGNORECASE) for pat in SHIFT_LEFT_KEYWORDS)

        if is_dev_request:
            reminder = (
                "[Shift-Left Quality Guardrail]\n"
                "1. Contract-First: Define Rust IPC structs & TypeScript interfaces before logic.\n"
                "2. TDD: Write failing tests in src/**/__tests__/*.test.ts or #[test] in Rust BEFORE writing code.\n"
                "3. Micro-Validation: Run `python .agents/scripts/verify_shift_left.py --changed` after every file edit.\n"
                "4. PrimeVue Foundation: No raw HTML <button>/<input>/<select>; use PrimeVue with v-tooltip.\n"
                "5. Boundary: No direct Tauri invoke or @tauri-apps/api in UI components/composables (use @/services/*)."
            )
            print(json.dumps({
                "injectSteps": [
                    {
                        "ephemeralMessage": reminder
                    }
                ]
            }))
        else:
            print(json.dumps({}))

    except Exception:
        # Never crash the agent loop
        print(json.dumps({}))

if __name__ == "__main__":
    main()
