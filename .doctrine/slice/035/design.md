# SL-035 Design — record-time stderr nudge for hidden thread memories

## Current vs target behaviour

**Current.** `memory::run_record` (`src/memory.rs`) writes one stdout line on
success (`Recorded memory <uid> [(<key>)]: <dir>`) and exits. A `--type thread`
record is indistinguishable from any other at the CLI, yet `thread_expiry`
(`src/retrieve.rs`, SL-008 D6) silently excludes it from `find`/`retrieve` until
it is `verified` ∧ `reviewed`≤14d. The agent learns nothing.

**Target.** When the recorded memory is a `thread`, after the success line, emit
one **stderr** advisory:

```
warning: a `thread` memory is invisible to find/retrieve until verified
(SL-008 D6). Verify it on a clean tree — `doctrine memory verify <ref>` — or it
surfaces only in list/show.
```

`<ref>` = the minted key if present, else the uid (both drive `verify`). Stdout
is unchanged (machine-readable success line preserved). Non-thread records are
untouched.

## Code impact

`src/memory.rs` only.

- **New pure fn** (impurity-free, per ADR-001 / the pure-imperative split):

  ```rust
  /// The record-time advisory for a freshly-minted memory. A `thread` is hidden
  /// from find/retrieve until verified (SL-008 D6 thread_expiry); every other
  /// type surfaces immediately, so returns `None`. `reference` is the verify
  /// handle (key if present, else uid). Pure — text in, text out.
  fn thread_hidden_notice(memory_type: MemoryType, reference: &str) -> Option<String>
  ```

  Returns `Some(msg)` only for `MemoryType::Thread`, with `reference` spliced in.

- **Shell wiring** in `run_record`, immediately after the success `writeln!`:

  ```rust
  let reference = key.as_deref().unwrap_or(&uid);
  if let Some(notice) = thread_hidden_notice(args.memory_type, reference) {
      writeln!(io::stderr(), "{notice}")?;
  }
  ```

  Mirrors the existing linked-worktree stderr warning (`run_record`, the
  `is_linked_worktree` branch) — same seam, same non-blocking posture.

## Trigger scope

Fires for **every** `--type thread` record, `--global` included: a global thread
is gated by `thread_expiry` identically (the gate keys on `kind`, not on
repo/anchor). `record` always scaffolds `unverified`, so the advisory is always
accurate for a thread; no need to read the scaffolded state back.

## Verification

- **VT — new unit tests on `thread_hidden_notice` (pure):**
  - `MemoryType::Thread` → `Some`, message contains the `<ref>` and `verify`.
  - a non-thread (e.g. `Pattern`) → `None`.
  - the `<ref>` splice uses key-when-present, uid-when-absent (test both).
- **Behaviour-preservation:** the existing `memory.rs` + e2e suites stay green
  unchanged. No read-path / `thread_expiry` edit, so the SL-008 retrieval suites
  are untouched by construction.
- The stdout success line is asserted unchanged (no golden churn for non-thread;
  a thread record adds a stderr line only).

## Decisions

- **D1 — stderr, not stdout.** The advisory is human guidance, not part of the
  record's machine-readable result; stdout stays parseable. Consistent with the
  worktree warning.
- **D2 — pure helper + thin shell.** Keeps the decision/text testable without a
  subprocess and honours the pure-imperative split. The shell does only IO.
- **D3 — no quiet/suppress flag in v1.** One line on stderr is cheap and the
  signal matters; a `--quiet` axis is scope creep (out of scope, slice-035.md).
- **D4 — wording fixed** (user-approved, warning-style, matches the worktree
  line's tone). Cites SL-008 D6 so the reader can trace the canon.

## Non-goals (from slice scope)

No `thread_expiry` / read-path change; no new flag; no stdout change; no
suppression flag. D6 behaviour stands.
