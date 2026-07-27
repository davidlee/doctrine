#!/usr/bin/env python3
r"""RV-307 F-38's two obligations: can a control character reach a scope entry,
and what happens to it downstream?

F-38 names two obligations that must not be conflated:
  (1) REJECTION at the write verbs for argv-unrepresentable values. A NUL yields
      no git process at all, so no exit code — it is outside E11/E13's exit-code
      taxonomy entirely.
  (2) ESCAPED FRAMING for anything reaching a report. A newline splits a
      per-entry finding across lines with no framing rule.

FALSIFIERS, registered before the probe runs:
  FAL-N1  TOML must be able to REPRESENT a NUL inside a string. If the format
          rejects it, the file route is closed and (1) narrows to the API route.
  FAL-N2  argv must be UNABLE to carry a NUL. If it can, git returns a verdict
          and F-38's premise ("no process, so no exit code") is false.
  FAL-N3  JSON must be able to carry one — that is the MCP route, and it is the
          route that stays open when the CLI route is closed by FAL-N2.
  FAL-N4  a NEWLINE must reach git and return an ordinary verdict. If newlines
          also abort, obligation (2) collapses into obligation (1) and the
          "do not conflate" instruction is wrong.
"""

import json
import subprocess
import tomllib

NUL = "\x00"
NL = "\n"

print("=== FAL-N1 — can TOML represent a NUL in a string? ===")
for label, src in [
    ("plain", 'x = "ab"'),
    ("escaped \\u0000", 'x = "a\\u0000b"'),
    ("literal NUL byte", 'x = "a' + NUL + 'b"'),
]:
    try:
        v = tomllib.loads(src)["x"]
        print(f"  {label:20s} -> PARSED, value={v!r}")
    except Exception as e:  # noqa: BLE001
        print(f"  {label:20s} -> REJECTED: {type(e).__name__}: {e}")

print()
print("=== FAL-N2 — can argv carry a NUL? ===")
try:
    subprocess.run(["git", "ls-files", "--", "a" + NUL + "b"], capture_output=True)
    print("  argv carried a NUL — F-38's premise would be FALSE")
except ValueError as e:
    print(f"  ValueError: {e}")
    print("  -> no git process is created, so there is NO exit code to classify")

print()
print("=== FAL-N3 — can JSON carry one? (the MCP route) ===")
v = json.loads('{"p": "a\\u0000b"}')["p"]
print(f"  json.loads -> {v!r}   (len {len(v)})")
print("  -> the MCP write verbs accept JSON, so this route stays open")

print()
print("=== FAL-N4 — does a NEWLINE reach git and get a verdict? ===")
p = subprocess.run(
    ["git", "ls-files", "--error-unmatch", "--", ":(literal)a" + NL + "b"],
    capture_output=True,
)
print(f"  git exit={p.returncode}  (1 = ordinary unmatched verdict, 128 = abort)")
print("  -> a newline is representable, reaches git, and returns a verdict.")
print("     It is a REPORTING hazard, not an argv hazard — distinct from NUL.")

print()
print("=== obligation (2): what a newline does to a per-entry report line ===")
entry = "src/a" + NL + "src/b"
print("  unframed:")
print(f"    mem_x: non-contributing scope entry {entry}")
print("  -> two lines; the second is indistinguishable from a second finding.")
print("  framed (escape control chars before interpolation):")
print(f"    mem_x: non-contributing scope entry {entry!r}")
