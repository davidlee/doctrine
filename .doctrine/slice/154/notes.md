# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design handover (2026-06-26) — context bootstrap for `/design`

Slice is scoped (`slice-154.md`) and in `design` status. Start by reading
`slice-154.md` in full, then this. Your job: `/design` → `/inquisition` until the
decisions lock. **Root-cause first; do not jump to a fix in the scope doc — it is
deliberately observable-only.**

### What this slice is

Close the two conformance-registry **population** leaks RFC-004 v0.1 (SL-147)
left. The registry is `.doctrine/state/slice/NNN/boundaries.toml` (runtime tier)
— one `[[boundary]]` row per landed phase — and it is the **actual-side input**
to `slice conformance`. Two landing paths feed it; both leak:
- **ISS-051 (solo path):** final phase gets no row. Seam:
  `state.rs::capture_phase_boundary` bound to `set_phase_status`.
- **ISS-052 (funnel path):** dispatch funnel never writes the registry. SHAs land
  only in the *dispatch ledger* (a different file). Seam: `dispatch.rs` integrate/
  land beat.

Scope is **registry-population only** (User decision). Stand-alone, references
**RFC-004** (NOT RFC-005).

### The axis correction that triggered this scope (don't regress it)

Pre-SL-152 framing was "claude arm vs subprocess arm; claude double-writes."
**That axis is dead.** SL-152 (closed) converged worker *creation* onto one seam
(`worktree fork --worker` via the `WorktreeCreate` hook). The correct axis for
**recording** is the **landing path** — solo (in-tree) vs funnel — not the
harness. See RFC-005 §Reconsideration (2026-06-25) + §H3.

### Two `boundaries.toml` files — do not conflate (this cost me a wrong frame)

| file | tier | what | hazard | owner |
|---|---|---|---|---|
| `.doctrine/dispatch/NNN/boundaries.toml` | committed | dispatch **ledger** | **ISS-039** (never committed to dispatch branch) | RFC-005 H3 — **OUT OF SCOPE** |
| `.doctrine/state/slice/NNN/boundaries.toml` | runtime | arm-neutral **conformance registry** | ISS-051/052 (population) | **THIS SLICE** |

`ledger.rs` owns the first (`record_boundary`, `.doctrine/dispatch/`); `state.rs`
owns the second (`capture_phase_boundary` → `record_source_delta`,
`.doctrine/state/slice/`). Shared row type: `boundary.rs::BoundaryRow`.

### Open root-cause questions FOR DESIGN (the hard part)

1. **Solo final-phase miss — timing vs missed transition?** Read
   `state.rs:410-532`. The binding stamps `code_start_oid` on the `in_progress`
   flip and records `(start, end=HEAD)` on the `completed` flip — so on paper the
   final phase *should* capture. Yet SL-147 PHASE-06 got no row. Two hypotheses:
   (a) `end=HEAD` is read at the `completed` flip but the phase's last commit
   lands *after* the flip (operator flips completed, then commits notes/dogfood)
   → either a short/empty range or, if start==end, a dropped row; (b) the final
   phase is flipped via a path that bypasses the binding (e.g. `/close`), so
   `set_phase_status` capture never runs. User's recollection was "we rely on the
   next phase verb to drive auto-capture" — reconcile that memory against the
   code (the code does NOT look like next-phase-driven). **Confirm empirically**
   before designing the fix.

2. **SL-153 is the sharpest reproduction — and it broke BOTH paths.** SL-153 is
   **mixed-mode**: P01/P02 landed solo (before the dispatch drive started at
   `ab2c642f`; see `slice/153/handover.md`), P03/P04 dispatched. The conformance
   registry was **entirely empty** — so the solo binding *also* failed for
   P01/P02, not just the funnel for P03/P04. Why did solo P01/P02 leave no row?
   This is a second, independent data point on the solo gap (objective 1) and the
   mixed-mode composition problem (objective 3). Worth a full post-mortem of
   SL-153's phase-status transition history.

3. **Mixed-mode composition (objective 3).** The two writers must compose with no
   gap and no duplication across the solo↔funnel boundary within one slice. The
   existing solo arm-guard (`state.rs:481` — skips capture when current branch ==
   `dispatch/NNN`) is the anti-double-record seam; check it holds under mixed
   mode.

4. **Fix altitude for ISS-052.** Scope doc leans "enforced funnel beat" over a
   read-time fallback (mirror the SHAs the funnel already has at integrate into
   the conformance registry). Confirm that's right vs. e.g. having `slice
   conformance` fall back to the dispatch ledger when the registry is absent
   (rejected in scope as papering over the write gap — but design should
   re-litigate if the funnel write proves expensive).

### Evidence already gathered (don't re-derive)

- RFC-004 Outcome (updated this session): prove-value MET. `slice conformance
  147` → undeclared 24 / undelivered 0 / conformant 7 after I bootstrapped
  PHASE-06 via `record-delta`.
- `slice conformance 153` now runs (I seeded selectors + bootstrapped all 4
  boundaries from the dispatch ledger + handover commit map). Result: conformant
  2 / undelivered 0 / undeclared 2 (`guard.rs`, a memory `.md`). That seeding is
  **manual** — exactly the toil this slice removes.
- SL-153 phase→commit map (verified linear `c371b839`→P01→P02→P03→P04):
  P01 `d3947526` (dep_seq.rs), P02 `ab2c642f` (spec.rs/guard.rs), P03 `71466d0d`,
  P04 `0cc4800c`.

### Code map (start here)

- `src/state.rs:410-532` — `set_phase_status` tail + `capture_phase_boundary`
  (solo binding; ISS-051). `record_source_delta` ~:616. Write path resolves the
  PRIMARY tree (`:574`).
- `src/dispatch.rs` — integrate/land beat (`integrate` ~:519/:1550 per SL-147
  affected-surface notes); where the funnel must mirror into the conformance
  registry (ISS-052).
- `src/ledger.rs:538` — `record_boundary` (dispatch ledger; the SHA source to
  mirror, read-side reuse — NOT the ISS-039 commit seam).
- `src/boundary.rs:16` — `BoundaryRow`.
- `plugins/doctrine/skills/dispatch{,-subprocess}/SKILL.md` — currently document
  a skippable orchestrator `slice record-delta` step; drop once the funnel beat
  is enforced.

### Constraints / canon

- **POL-002** — platform independence; recording rides doctrine-owned contracts
  (recorded SHAs, the funnel), never host commit conventions. The solo arm-guard
  keys on the doctrine-owned branch name `dispatch/NNN`, not host convention —
  keep it that way.
- **ADR-001** layering: `boundary.rs` is a leaf; `state.rs`/`ledger.rs` engine;
  keep git/disk in the shell, pure row logic in the leaf.
- `just check` green before every commit; clippy plain (no `--all-targets`).
- `record-delta` (`slice record-delta`) STAYS as the manual escape hatch — this
  slice removes the *need* for it on a normal slice, not the verb.

### Relations

references→RFC-004 (concerns); related→ISS-051, ISS-052. Non-goals: ISS-039,
RFC-005 H1/SL-152 creation, selector-authoring adoption (follow-up).
