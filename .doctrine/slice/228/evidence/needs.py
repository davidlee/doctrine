#!/usr/bin/env python3
"""Extract the 'situations encountered' ledger for OQ-6 (SL-228 PHASE-07 EX-3).

For each benchmark round: every action the blind subject took, every refusal or
error it met, and every passage where it reasoned about dispatch mechanics. This
is the evidence base for EX-3's second question — did the subject actually NEED
the fact a memory carries, or did it rediscover it the hard way?
"""
import json
import re
import sys

FRICTION = re.compile(
    r"refus|denied|error|failed|fatal|not found|cannot|unable|unknown-|"
    r"undeclared|not-landed|stale|halt|no such",
    re.I,
)
DELIBERATION = re.compile(
    r"I need to|I should|let me check|figure out|not sure|unclear|"
    r"turns out|apparently|so the|which means|the problem is|stuck|"
    r"doesn't exist|no longer|instead of",
    re.I,
)


def squash(s, n):
    return re.sub(r"[ \t\n\r]+", " ", s)[:n]


def run(path, label):
    print(f"\n{'=' * 78}\n## ROUND {label}  ({path})\n{'=' * 78}")
    step = 0
    pending = {}
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        typ = e.get("type")
        content = e.get("message", {}).get("content", []) or []
        if typ == "assistant":
            for b in content:
                if b.get("type") == "text":
                    t = b.get("text", "")
                    for para in t.split("\n\n"):
                        if DELIBERATION.search(para) and len(para) > 60:
                            print(f"     THINKS: {squash(para, 420)}")
                elif b.get("type") == "tool_use":
                    step += 1
                    nm = b.get("name", "").replace("mcp__doctrine__", "MCP:")
                    inp = b.get("input", {})
                    arg = inp.get("command") or inp.get("file_path") or json.dumps(inp)
                    print(f"{step:4}. {nm} | {squash(str(arg), 200)}")
                    pending[b.get("id")] = (step, nm)
        elif typ == "user":
            for b in content:
                if b.get("type") != "tool_result":
                    continue
                c = b.get("content")
                if isinstance(c, list):
                    c = " ".join(x.get("text", "") for x in c if isinstance(x, dict))
                c = str(c or "")
                if FRICTION.search(c) or b.get("is_error"):
                    s, nm = pending.get(b.get("tool_use_id"), (step, "?"))
                    print(f"     FRICTION <-{s} {nm}: {squash(c, 500)}")


for p in sys.argv[1:]:
    run(p, p.split("/")[-1].replace(".jsonl", ""))
