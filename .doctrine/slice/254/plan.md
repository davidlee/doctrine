# Implementation Plan SL-254: Collapse dispatch onto one subprocess arm

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Nine phases. One of them adds code, one drives a live dispatch, one rewrites
governance, and the other six delete. That ratio is the slice: `DEC-202`
falsified the premise the claude arm was built on, and the work is removing what
that premise justified rather than building a replacement — the replacement
already runs, on codex and pi.

The plan's whole shape comes from two constraints that are not the implementer's
to choose.

**Re-home before delete** (`DEC-206`, design §4). The four jail primitives move
to `jail.rs` while the change is behaviour-preserving and the existing suite is
its proof. Design §9.1 calls this the cheapest point at which the
behaviour-preservation gate does useful work, and §4 states the ordering as a
design fact rather than a preference. It is `PHASE-01` and it is the only phase
whose exit criterion is a *negative*: zero test edits.

**Every phase boundary is a green tree.** `just gate` runs clippy at zero
warnings, and this project's `dead_code` exposure is the reason the phase cut
falls where it does rather than where the design's file list suggests. Deleting
a `pub(crate)` item's last caller in one phase and the item in the next leaves a
red tree in between. That is not a stylistic preference; it is what makes a
phase independently landable.

## Sequencing & Rationale

**Why the wall splits into two phases (`PHASE-03`, `PHASE-04`).** The obvious
cut is one big deletion. The census showed why that is wrong: `pretooluse.rs` is
the sole caller of roughly 300 production lines in `jail.rs`, so the wall's
deletion drags a third of that module with it. Cutting nomination out first
(`PHASE-03`) separates cleanly — it takes `PRIVILEGED_AGENT_TYPES`,
`is_privileged_agent_type`, `decide_agent`, `decide_workflow` and
`resolve_target`'s nomination leg, and it is what forces doctor check #10 out,
which does not compile once `SEAM_REGISTRY` and the `SubagentStart` matchers go.
What is left in `PHASE-04` is the wall proper and the rest of the decision layer.
Two reviewable phases instead of one 2,500-line one, each green.

**Why the script comes before any deletion (`PHASE-02`).** Substitution, not
construction (design §4). The confined spawn path is what makes every subsequent
deletion safe, so it lands first and additively — nothing is removed in that
phase, and the pi arm is untouched. It also has to be first for a mechanical
reason: `PHASE-05`'s identity collapse depends on something setting
`DOCTRINE_WORKER=1`, and that something is `PHASE-02`'s prefix.

**Why identity comes after the wall (`PHASE-05`).** The marker and the wall are
entangled through `subagent.rs`, which holds the stamp, `verify-worker` and the
nomination verbs in one file. Splitting by *subcommand* rather than by file lets
each phase stay green: `PHASE-03` takes the nomination half, `PHASE-05` takes the
stamp and `verify-worker` halves and deletes the file.

**Why governance is late and large (`PHASE-08`).** `VH-2` requires the target set
to be re-derived *at the REV phase*, from the corpus as it then stands — not read
off design §3.1's table. Running it last means the corpus is corrected against
what shipped rather than what was planned. It is the larger half of the change by
the design's own accounting.

**Why the live dispatch is a phase and not an audit step (`PHASE-09`).** Design
§9.3 is blunt that nothing in the unit suites exercises a real `claude -p` under
bwrap, and `A2`'s residue is provisioning, which fails at runtime or not at all.
Driving one real phase end to end is work, not a check, and its evidence has to
land in an authored sink rather than the gitignored scratchpad (`DEC-212`).

## Notes

### Counts in the design are floors, and the plan phase found two more

`R8` records that this design's surveys under-count — six times at design close,
always low. Re-grepping every concrete premise before scaffolding (which the
`/plan` process requires) confirmed **every** path, symbol, config key and script
reference the design names still resolves. Nothing was stale. But two counts were
low again, and both are written into the phase criteria rather than left for an
implementer to trip over:

1. **`tests/e2e_claude_install.rs` is more than one assertion** (`PHASE-04/EX-6`).
   The design says "the exact hook-count assertion moves by six". In fact the
   test *function name* encodes the count
   (`install_wires_eleven_hook_entries_across_five_events`), the events list
   drops two entries because `SubagentStart` and `SubagentStop` each held exactly
   one hook and disappear entirely, and a separate block asserts "SubagentStart
   carries nominate, NOT the retired SL-152 stamp hook". Four edits, not one.
   The installed set goes from eleven entries across five events to **five across
   three**.

2. **`spec-021.toml` has two dangling `[[source]]` anchors, not one**
   (`PHASE-08/EX-8`). `R7` names `:30` (`dispatch-agent/SKILL.md`) as "the only
   true dangling anchor". `:34` points at `dispatch-subprocess/SKILL.md`, which
   the same merge deletes. `spec validate` catches neither, which is exactly why
   `R7` exists.

Both are under-counts of the kind `R8` predicts, found by the mechanism `R8`
prescribes. Neither invalidates a design premise, so neither warranted reopening
the design — the plan is where the re-derivation is supposed to land.

### What the critical pass changed

The first draft of this plan had four defects, all found by checking its own
assumptions against the tree rather than by re-reading it:

1. **`PHASE-01` was unsatisfiable as written.** Its exit criterion demanded zero
   edits to any `#[cfg(test)]` block, but `pretooluse.rs` holds two unit tests
   for `write_seatbelt_profile` (`:1083`, `:1100`) that must relocate with their
   subject. The criterion now distinguishes the two cases: `tests/` takes zero
   edits, unit tests move verbatim, and a relocated test whose *assertions*
   changed is the failure signal.
2. **`PHASE-03` would have ended on a red tree.** `tests/e2e_claude_install.rs`
   asserts the hook total, and `PHASE-03` takes it from eleven to nine — so the
   test has to move in that phase, not wait for `PHASE-04`. Leaving it would have
   broken the one property the whole phase cut exists to preserve. `EX-4b` now
   carries it, and `PHASE-04/EX-6` is explicitly the *second* cut.
3. **`src/main.rs` was under-attributed.** Its `mod write_class_tests` (`:326`)
   is a table-driven exhaustiveness check naming `worktree_nominate_is_hookmint`
   (`:650`), `worktree_marker_stamp_subagent_is_hookmint` (`:627`) and a
   `MarkerClear` assertion (`:616`). The design said "drop the write-class tests
   for the deleted verbs" and was right; the plan had folded it into
   `guard.rs`. Split across `PHASE-03/EX-4` and `PHASE-04/EX-8`.
4. **`PHASE-06/VT-2` pointed at the wrong file.** `CheckKind` lives in
   `src/verify.rs`, which owns the enum and its resolution; `src/dispatch.rs`
   never names it. The mandate would have been `UNCHECKABLE` — the exact inert
   state `IMP-209` warns about — while looking well-formed.

Two of the four (1 and 2) would have surfaced as a failed phase mid-execution.
The other two would have passed silently while verifying nothing.

Also settled: `.claude/settings.json` is tracked but is NOT hand-edited. It is
reconciled by `doctrine install` against `boot.rs`'s `HookSpec` registry (design
§5.6), so both phases that touch the hook set edit the registry and re-run
`install`.

### The census facts the design's file list does not carry

Three consumers were absent from design §5.6 until the sixth sweep, and they are
load-bearing for phase sizing rather than incidental:

- **`src/commands/guard.rs`** holds the write-class registry, not `src/main.rs`.
  Its `Nominate`/`Denominate` arms (`PHASE-03/EX-4`) and its
  `Marker { stamp_subagent: true }` arm plus the `WriteClass::MarkerClear` class
  (`PHASE-04/EX-4`) do not compile once those `Command` variants go.
- **`justfile:37`** carries a marker-file leg in `validate`'s worker-context skip
  (`PHASE-05/EX-5`). Non-Rust, so no symbol census reaches it; it surfaced only
  by grepping the literal path string.
- **`jail.rs`'s decision layer** (`PHASE-04/EX-2`) — the ~300 lines and ~30 unit
  tests that orphan when `pretooluse.rs` goes.

### What the mandates can and cannot say

A `VT` mandate asserts *presence*: substrings that must appear in one named file
after the phase lands. This slice is mostly deletion, and absence has no direct
spelling. So each deletion phase anchors on the surviving or renamed artefact
that can only exist once the deletion happened — a renamed test function, a
re-cut assertion, the surviving closure of `jail.rs`. Where a claim is genuinely
about absence across many file types (`PHASE-05/VA-1`'s marker path,
`PHASE-06/VA-1`'s config key), it is a `VA` checked by grep at agent altitude,
because dressing it as a `VT` would make it pass vacuously — the same fail-open
shape `DEC-212` is deleting elsewhere.

### Deferred, with cards

`IMP-428` (hardening delta), `IMP-429` (the prefix's permanent home and macOS
parity), the narrowed `~/.claude` mount set, and `SL-255` (clone provisioning)
are all real and all out of scope. Design §4: "one thing at a time" — each was
declined on scope, with a card, not overlooked. `PHASE-09/VA-2` carries the one
piece of that which must not be silently dropped: if no mac is available, the
Darwin parity claim is recorded as **unverified** rather than left to read as
verified.
