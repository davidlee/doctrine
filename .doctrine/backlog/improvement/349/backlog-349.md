# IMP-349: Single-source the .doctrine/state/slice path root

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The gitignored runtime slice-state root `.doctrine/state/slice` has one named
constant and **nine** bare-literal spellings beside it — three in production
code, six in tests, three of those inside `src/state.rs`'s own test module.
STD-001 (no magic strings — single-source named constants) is not satisfied.

Measured at `edge`, 2026-07-29 (`rg -n '\.doctrine/state/slice' src/`, comments
excluded):

| site | form | |
|---|---|---|
| `src/state.rs:33` | `const STATE_SLICE_DIR: &str = ".doctrine/state/slice"` | the intended source — **private** |
| `src/kinds/mod.rs:358` | `state_dir: Some(".doctrine/state/slice")` | production |
| `src/lazyspec.rs:1600` | `.join(".doctrine/state/slice")` | production |
| `src/dispatch.rs:8300` | `.join(".doctrine/state/slice")` | production |
| `src/state.rs:2059,2245,2250` | `.doctrine/state/slice/147/boundaries.toml` | test, ×3 |
| `src/integrity.rs:608` | `.doctrine/state/slice/001/phases/phase-01.md` | test |
| `src/doctor_checks.rs:1346` | `.doctrine/state/slice/001/phase-01.md` | test |
| `src/worktree/allowlist.rs:308` | `.doctrine/state/slice/029/phases/phase-01.md` | test |

`STATE_SLICE_DIR` is private to `src/state.rs`, so no other module *can* ride it
— which is why the literal was retyped rather than imported. The sharpest one is
`kinds/mod.rs:358`, where the kind registry's `state_dir` field is arguably the
more canonical home for this path than `state.rs`'s constant. That three of the
duplicates sit in `state.rs`'s own tests, a few hundred lines from the constant,
shows the constant was not reachable *or* memorable even at home.

**Why this is worth doing.** These will drift. A change to the runtime state
layout has to find ten spellings with a grep that also matches doc comments, and
a missed one fails at runtime in a gitignored tree — the tier where a wrong path
is hardest to notice, because the symptom is silently absent state rather than a
diff.

## How this surfaced

SL-233 PHASE-03's `VA-3` mandated `rg -n '\.doctrine/state/slice' src/ --glob
'!src/state.rs'` return **zero hits**, and its `EX-8` states "a second
`.doctrine/state/slice` literal anywhere is the violation this criterion exists
to prevent". Both were authored believing `src/state.rs` was already the sole
source. It is not, and has not been for some time — the criterion described a
condition that never held at head.

SL-233 PHASE-03 is **not** chartered to fix this, and did not. Its `VA-12`
(appended 2026-07-29 under the owner's ruling) narrows the sweep to that phase's
own paths and defers the corpus-wide duplication here, rather than silently
absorbing it or silently fixing it out of scope. PHASE-03 adds
`pub(crate) fn design_snapshot_path` to `src/state.rs` deriving from the private
constant, so it adds no eighth literal.

## Shape of the fix

Decide the canonical home first — `src/kinds/mod.rs`'s `state_dir` is the
registry-level answer and `src/state.rs`'s `STATE_SLICE_DIR` is the module-level
one; they should not both exist. Then make it reachable (`pub(crate)`) and
retire the literals, tests included. Small and mechanical once the home is
chosen; the choice is the only real decision.

Relates to SL-233 PHASE-03 EX-8 / VA-3 / VA-12.
