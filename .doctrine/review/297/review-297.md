# Review RV-297 — reconciliation of SL-204

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Subject / surface.** SL-204 was driven by `/dispatch` (4 phases, all
`completed`). The implementation is the immutable evidence bundle on
`refs/heads/review/204` (tip `256f34bbf`, 50 files, +1212/−892) — **not landed on
edge/main**. Audited against that bundle in a detached worktree at
`/workspace/sl204-audit` (review/* is an R2 evidence ref; no candidate interaction
branch was published, and none is needed to *assess*). `web/map/dist` was seeded
into the worktree before building (known dispatched-slice audit gotcha —
`mem_019f4c64…`); its absence is an environment artifact, never a regression.

**Design invariant this slice is held to.** Pure structural inversion, *zero
validation-behaviour delta* — the existing validation + entity suites are the
behaviour-preservation proof (design §1, VT-B / EX-4). Any observable behaviour
change is a finding.

**Lines of attack.**
1. **Behaviour-preservation gate** — full `cargo test` + `cargo clippy --workspace`
   green *unchanged* on the bundle; `architecture_layering` green with
   `integrity = "engine"`, the 13 up-reaches gone, the 4 back-cycles broken, and
   the command tangle ratcheted to the measured value (baseline `command = 76`).
2. **Conformance** — `slice conformance` reads the *edge* selector registry
   (stale: the bundle's selector declarations are un-landed), so its
   `undeclared(4)` is un-landed authored state, not drift. The load-bearing signal
   is the bundle's own registry + `undelivered`.
3. **Design fidelity** — as-built vs `design.md` D2/D3/D4 and §5. The notes flag a
   known D2 premise failure (the `listing::canonical_id` route was cycle-infeasible;
   authority relocated into `kinds`). Probe whether design prose still tells the
   truth.
4. **Scope discipline** — off-§5 touches (`listing.rs`, `memory.rs`, `compare.rs`,
   `lazyspec.rs`) — benign retargets/collateral, or undocumented creep?
5. **Integration reality** — the bundle is ~2 weeks stale and unintegrated; trunk
   moved 22 commits past the prepared base. Zero `src/` overlap and no new callers
   of the relocated symbols → integration is clean. The one collision (edge ISS-235
   vs bundle ISS-235) was resolved pre-audit by reseating edge's → ISS-236.
   `refresh-base` + re-prepare remains for reconcile/close.

## Synthesis

**Verdict: the implementation is faithful and the central invariant holds; all
gaps are benign and reconcile to prose/registry, not code.**

SL-204 delivered the designed inversion. The kind-identity `Kind` struct, the
`KINDS` table + `KindRef`, and the five ref resolvers relocated to the leaf
`kinds/` umbrella (`mod.rs` + `resolve.rs`); `entity::Kind` re-exports the leaf
type so ~200 field accesses and every `entity::Kind` import compiled unchanged;
`GovKind` became the dual-surface descriptor borrowing the leaf identity and
carrying the scaffold that D3 evicted from `Kind`; the three panic-stub scaffolds
are gone. `integrity` dropped all 13 kind-module up-reaches and re-tiered
command→engine, the four back-cycles (backlog/rec/review/revision) broke, and the
command tangle ratcheted to a measured **76** (design D6 deferred the number to
execute-time; ejection from the core SCC cascaded well past the four nominal
2-cycles, exactly the RSK-227/SL-203 precedent). The behaviour-preservation gate —
the whole justification for calling this "pure structural inversion" — is green
unchanged: full suite 0-failed, clippy zero-warnings, `architecture_layering`
17/0 (F-1).

**Standing risks / tradeoffs consciously accepted.**
- **One real design delta (F-3):** design D2's `listing::canonical_id` routing was
  cycle-infeasible (leaf 3-cycle `kinds→listing→tag→kinds`). The as-built relocates
  the `canonical_id` authority *down* into `kinds` and delegates `listing→kinds` —
  a strictly better placement (the format authority now sits with its inverse
  `parse_canonical_ref`). Sound; design prose is merely stale.
- **Documentation lag (F-2, F-4):** the selector registry and design §5 over-declare
  three untouched files and omit four legitimately-touched ones. No drift — the
  conformance signal is purely un-landed authored state read against the edge
  registry. Reconcile trims the registry (the load-bearing artefact) and mirrors §5.
- **Authorized collateral (F-5):** a non-hermetic `vt9` test (`memory.rs`) was made
  hermetic mid-dispatch to unblock the worker gate — plan-authorized (EX-6),
  recorded, zero product-behaviour delta.
- **Integration debt (F-6):** the slice sat unintegrated for ~2 weeks while trunk
  advanced 22 commits. The audit de-risked the landing: zero `src/` overlap, no new
  callers of relocated symbols, and the sole id-collision (ISS-235) already reseated
  to ISS-236 (c20b56bc). Landing is mechanical reconcile/close work.

No blocker; every finding terminal-verified. Cleared to reconcile.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md D2 + EX-5 cross-ref (F-3):** replace the "route id-formatting through
  `listing::canonical_id`" mechanism with the as-built: the `canonical_id` format
  authority lives in `kinds` (`kinds::canonical_id`, inverse of
  `parse_canonical_ref`, placed below `tag`); `listing::canonical_id` delegates up,
  preserving the ~30-module caller seam. Note the rejected route's infeasibility
  (leaf 3-cycle `kinds→listing→tag→kinds`).
- **Selector registry — the load-bearing conformance fix (F-2):** run
  `doctrine slice selector rm SL-204 src/commands/cli.rs`,
  `doctrine slice selector rm SL-204 src/commands/doctor.rs`,
  `doctrine slice selector rm SL-204 tests/architecture_layering.rs`. These three
  are declared-but-undelivered; removing them is what makes `slice conformance`
  clean (VA-2 / VA-A). `design.md §5` is only the human mirror.
- **design.md §5 code-impact table (F-4, mirrors F-2):** add rows for
  `src/listing.rs` (canonical_id delegation), `src/memory.rs` (vt9 hermeticity
  collateral, EX-6), `src/commands/compare.rs` and `src/lazyspec.rs`
  (`integrity::`→`kinds::` retarget / GovKind `.kind` deref); remove the
  cli.rs/doctor.rs/architecture_layering.rs rows in step with the selector rm.

### Governance/spec (REV)

- **None.** No ADR, policy, standard, spec, or requirement change. ADR-001 layering
  is data (`layering.toml`), already updated in the bundle and validated by
  `architecture_layering`; it is not a governance-prose edit.

### Integration / close (not a reconcile write-surface — for /close, F-6)

- Promote `edge`→`main` (`git fetch . edge:main`), then land the bundle: the
  dispatch tooling wants `doctrine dispatch refresh-base --slice 204` → re-prepare →
  integrate onto `main`. Integration verified clean (zero `src/` overlap, no new
  callers). The ISS-235 collision is already dissolved (edge reseated → ISS-236,
  c20b56bc); the bundle's `.doctrine/` — including its own ISS-235 — lands verbatim.
- Re-run `doctrine check gate` on the landing tree at close (fresh-binary rule) —
  the layering/tangle assertions must exercise the real authored corpus.

### Sequencing caveat (reconcile ↔ close ordering)

- The **selector rm (F-2)** edits `slice-204.toml`, which the **bundle also
  modified** (+17 selector rows). Apply the rm **after** the bundle integrates, on
  the landed tree — doing it on edge now would collide with the bundle's
  `slice-204.toml` at integration. The **§5 edit (F-4)** touches `design.md`, which
  the bundle did **not** change (design locked pre-dispatch), so it is conflict-free
  either side; the **D2 edit (F-3)** likewise. Recommended order: reconcile the
  `design.md` prose (F-3, F-4) now → close lands the bundle on main → apply the
  selector rm on the landed tree → `doctrine check gate`.
