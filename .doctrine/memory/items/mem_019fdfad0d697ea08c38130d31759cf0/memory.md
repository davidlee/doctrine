`tests/e2e_claude_install.rs` carried three assertions of the form

```rust
assert!(event_entries(&settings, "WorktreeCreate").is_empty(),
        "no WorktreeCreate hook settings-wired (ships via plugin)");
```

with `event_entries` returning `Vec::new()` for an unreadable file. They passed
from SL-152 PHASE-06 until SL-250 PHASE-04 — but **not for the stated reason**.
The settings file was never written at all, so the assertion never discriminated
between "the plugin owns this hook" and "nothing ran".

## The mechanism underneath

`doctrine install` resolved its boot leg with
`crate::boot::resolve_harnesses(&[], root)` — an **empty explicit list**, in both
`run_forward_steps` and `print_forward_summary`. `resolve_harnesses` honours an
explicit list and otherwise auto-detects `.claude/` / `.codex/` / `AGENTS.md` on
disk. So `--agent claude` was silently discarded, and in a fresh project
`.claude/` does not exist at that moment (the agent-def leg creates it *later* in
the same run). Result:

```
  boot         (no harness directories detected — skipped)
```

`doctrine install --agent claude` wired no `@`-import, no hooks, no `.mcp.json` —
and said so only in a line that reads like routine detection output. Fixed in
SL-250 PHASE-04 by `install_harnesses` / `boot_harness_names`: an explicit
`--agent` is a directive (honoured with no harness dir present, and honoured when
it implies none), `pi` maps onto the codex arm, no flags still means detection.

## How to apply

- **An `is_empty()` / `!contains()` assertion needs a positive control in the same
  test** — something the same run *did* write. Without one it cannot tell absence
  from "the code path never executed". Sibling of
  [[mem_019fbe0509397ea197c772a6864e3249]] (missing-file RED is a weak negative
  control) and [[mem_019fa18161f47651af7687d8dccbbc67]] (a negative grep is
  untrustworthy without a positive control) — same defect class, three surfaces.
- When a test's *comment* explains a pass by a mechanism (here: "ships via
  plugin"), check that the mechanism is what the assertion actually observes. A
  rationale in a comment is not evidence; it is the thing most likely to be stale.
- Suspect any e2e that drives a multi-leg verb in a **bare** tempdir: legs that
  self-skip on filesystem detection see a project that does not exist yet, and
  ordering within the run decides what they see.

Related: [[mem.pattern.boot.test-exec-is-not-doctrine-owned]] — the other way an
SL-250 fixture read as a clean pass while exercising the wrong branch.
