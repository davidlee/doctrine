# ISS-220: vt9 memory-surface test env-sensitive: fails when CLAUDE_PROJECT_DIR resolves an unmasked doctrine root

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

> **ISS-235 folded in here 2026-07-29** — same test, same cause, same false-red.
> ISS-235 (created 2026-07-24, "vt9 surface test non-hermetic on ambient
> CLAUDE_PROJECT_DIR") was an independent capture of this defect from a different
> slice's funnel; it is closed `duplicate` and its distinct content is merged
> below. Two captures three weeks apart is itself the signal that this class of
> false-red is hard to attribute from inside a worker gate.

## The defect

`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing`
(src/memory.rs) asserts "no discoverable root ⇒ emit nothing", but
`discover_surface_root` falls back from the (deliberately bogus) payload `cwd`
to the **process env** `CLAUDE_PROJECT_DIR`. The test never isolated that env
var, so its premise silently depended on the runner's environment: the fallback
resolved a real root, surfaced from that root's **live** memory corpus, and the
non-empty output panicked the assert.

Whether it fired was a function of two ambient facts, neither of them the
delta under test — which root the env var named, and whether that root's runtime
seen-set happened to already contain the memory vt9's input would surface
(session `s9`, changed `src/x.rs`).

## Both captures

| | ISS-220 | ISS-235 |
|---|---|---|
| observed | 2026-07-06, SL-206 PHASE-11 funnel | 2026-07-24, SL-204 PHASE-04 |
| context | `worker_commit` gate suite, env set from hook/subagent | claude-arm worker fork gate inheriting harness env |
| confirmed pre-existing | yes — base `7dcfed82`, untouched tree | yes — passes in a bare env, reds in-fork |
| case note | — | RFC-011 `SL204-a15d-P04-vt9-gate-falsered` |

Both landed the same consequence: `commit-gate-red` on a delta that never
touched `memory.rs`, blocking the claude dispatch arm.

ISS-220's original note recorded that the test "passes at `/workspace/doctrine`
only because a stale probe artifact `.doctrine/state/mem-surface-seen-s9.txt`
suppresses the surfacing — masked, not correct."

## Fix — landed, verified

SL-204 PHASE-04 fixed it opportunistically in that phase's worker delta, taking
the first of the two directions ISS-220 proposed. `src/memory.rs:11220`:

```rust
fn vt9_no_discoverable_root_emits_nothing() {
    let rootless = tempfile::tempdir().unwrap();
    let raw = stdin_read(rootless.path(), Some("s9"), None, "src/x.rs");
    ...
```

The tempdir **canonicalizes successfully**, so `discover_surface_root`'s
`.or_else()` env arm is never evaluated (`src/memory.rs:10635-10638`) and
`find_from` finds no marker above `/tmp` ⇒ `None`. Hermetic by construction, not
by masking: the seen-set is never read because no root is ever found.

Verified 2026-07-29 on `edge` (`45922945a`) — and deliberately verified in the
**unmasked** scenario ISS-220 named, since this tree still carries the masking
`mem-surface-seen-s9.txt`:

```
$ CLAUDE_PROJECT_DIR=/home/david/dev/doctrine/.worktrees/fable-loop \
    cargo test --bin doctrine vt9_no_discoverable_root
test memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing ... ok
```

That root is a live doctrine root with **no** `s9` seen-set, so a surviving env
dependence would have reddened. The full `ambient_surface` module is also green
(15 tests) with the env var pointed at the primary root.

## Residuals — why this stays open

1. **The env fallback has no test at all.** `ENV_PROJECT_DIR_SURFACE` has exactly
   two sites in `src/memory.rs` — the const (`:10516`) and the fallback
   (`:10636`). No test exercises or neutralizes it. vt9 no longer *reaches* it,
   which is a fix to one test, not a guard on the seam: nothing stops the next
   ambient sibling from regressing into the same ambient dependence. ISS-235's
   ask — sibling ambient tests should pin an empty corpus or inject the env
   lookup per the pure/imperative split — is unmet.
2. **Stale seen-set artifacts.** ISS-220 asked for `mem-surface-seen-s9.txt` to
   be cleaned from the primary tree's `.doctrine/state/`. It is still there
   (2026-07-24), alongside ~170 sibling `mem-surface-seen-*.txt` files going
   back to 2026-07-10 and a 242 KB `mem-surface.log`. Runtime tier, so
   disposable by definition — but nothing prunes it, and its presence is what
   masked this very defect. Sizing/retention is the actual open question.

ISS-235's third ask — that `worker_commit`'s gate run the funnel's B-vs-S
differential rather than a bare pass/fail, so ambient and pre-existing reds
cancel — is **not** carried here. It is already homed at **IMP-194**
(`cluster:worker-gate`), whose F-3/S1 analysis names this exact failure class:
"a new test failure is indistinguishable from pre-existing/env". Folding it in
again would be a third capture of one signal.

Related: `mem.pattern.dispatch.worker-commit-stale-path-false-red` (the other
known `worker_commit` false-red trigger — a stale validation binary; this issue
is the distinct env-sensitive-test trigger). [[IMP-194]] holds the differential
gate that would have cancelled both without attribution work.
