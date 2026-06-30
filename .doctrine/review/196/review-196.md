# Review RV-196 — reconciliation of SL-178

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-178 (close drift-discharge
legibility: richer error + skill recipe + shipped memory). All three phases
landed (PHASE-01 `326ebd75`, PHASE-02 `253d3bd8`, PHASE-03 `f7bf67f7`).

**Surface reviewed:** primary `edge` tree (solo, not dispatched). Bootstrapped
the missing PHASE-02/03 source-delta rows (`record-delta`) — only PHASE-01 had
auto-bound, so the registry read partial (the F-2 backstop). Re-ran conformance
against the complete registry.

**Lines of attack:**
1. **Behaviour-preservation gate** — the close-gate refuse/pass must be unchanged
   (design §3, R2). VT-1 (both phases), `check gate`.
2. **POL-002 / ADR-002** — no shipped artefact references host-project-local
   state; the shipped master carries the global-orientation INV signature.
   VA-2 body grep, VT-2 discoverability.
3. **Design-vs-as-built drift** — carry the phase-sheet findings (F-A..F-E) into
   the ledger: the supersede mechanism, key-resolution semantics, debug-embed
   timing, the new uid, and the authored-skill source path. Each is design.md /
   plan.toml prose that diverged from what executed.
4. **Conformance algebra** — undeclared/undelivered cells as leads.

**Invariants held:** ADR-005 tiering (one canonical source, others point);
STD-001 single const; ADR-001 (error stays in the slice shell).

## Synthesis

**Closure story.** SL-178 delivered all three legibility surfaces and the
behaviour-preservation gate held throughout. The close-gate refuse/pass logic is
unchanged (R2); only the error *payload* and `undischarged_drift`'s return type
moved. Evidence is green across the board:

- **VT-1** (both phases) PASS via `slice verify-vt 178` — the enriched error
  names each undischarged REQ with `authored: <status>`, the condensed (a)(b)(c)
  accept-REC predicate, and the `memory show <const>` pointer (asserted on
  substrings per design F-3/F-4, not the exact copy).
- **VT-2** PASS — the shipped master `mem.pattern.doctrine.close-drift-discharge-rec`
  (uid `mem_019f176f…`) is embedded, lints clean (INV signature + scope floor),
  and is discoverable via `memory find` after sync.
- **VA-1** — the `/close` skill (`plugins/doctrine/skills/close/SKILL.md:126`)
  carries the drift-discharge subsection pointing at the key.
- **VA-2** — no shipped artefact references host-project-local state: the error
  literal uses the const (no literal uid); the master body is scrubbed (F-1
  cross-ref genericized to prose, F-2 `ISS-006` → prose), and the only concrete
  ids are the explicitly-framed Doctrine-development worked example (the one
  conscious POL-002 tolerance, RV-195 F-2).
- `doctrine check gate` green; clippy zero-warning.

No blocker raised — the close-gate (D-C9b) will admit `audit → reconcile`.

**The standing theme: design prose lagged a hard-won execution truth.** Five of
six findings are design.md / plan.toml / selector defects where the artefact
described a mechanism that does not exist or a path that is gitignored — the code
and the as-built are correct, the canon is not. The deepest pair (F-1, F-2) is a
chain RV-195 could not have caught: RV-195 F-5 correctly killed the
hand-rolled-signature parallel-impl and prescribed `memory record --global` +
`memory status superseded`, but the supersede leg turned out *itself*
unimplementable — `superseded --by` resolves successors against `items/` only, and
key→uid resolution ignores status, so retiring a capture demands physically
dropping the items/ alias symlink. This was discovered only at execution and
resolved via a user-approved `/consult`. The audit's job here is to write that
truth back into the design so the next reader inherits the real mechanism, not the
plausible-but-wrong one.

**Tradeoffs consciously accepted.** (1) The worked example retains historical
Doctrine ids, explicitly framed as illustration — POL-002 tolerance per design
§5.4. (2) F-6: design-target selectors anchor on the stable key-alias and omit the
test seam (`src/corpus.rs`) and mint-mechanism churn — tolerated as selector
hygiene, not scope creep.

**Standing risk:** none material. The shipped key, the Fix-1 const, and the skill
pointer all spell the same string (R4 guarded by VT-1 + VT-2). The interim-release
hazard (R5, P1 ahead of P2) is moot — landing order PHASE-01 → {02,03} held.

## Reconciliation Brief

All findings are per-slice artefact corrections (design.md / plan.toml /
slice-178.toml). **No governance/spec (REV) surface is touched** — no ADR,
policy, standard, spec, or REQ changes. The implementation is correct as landed;
only the authored prose and one selector need to catch up to the as-built.

### Per-slice (direct edit)

- **design.md D3 + §5.4 step 3** (F-1): replace the `memory status superseded --by
  <new-uid>` retire mechanism with the as-built drop-items/-key-alias-symlink
  (`git rm`) + `memory status archived` on the capture. Align with §5.3's
  already-correct "removed" outcome. State the *why*: a shipped master under
  `memory/` cannot be a supersession successor (resolver is items/-only).
- **design.md §5.4 step 3** (F-2): add that key→uid resolution via the items/
  alias symlink ignores memory status, so physical symlink removal is what
  re-points `<key>` to the master — the reason the retire step drops the symlink.
- **design.md §5.3 + §5.4** (F-3): correct the "single physical home
  memory/mem_019f075f…" / "uid reuse … no supersede chain" prose to the fresh uid
  `mem_019f176f71537d12b1b09826a003a602`; drop the uid-reuse claim (consistent
  with revised D3).
- **design.md §10 F2** (F-4): correct to "debug-embed embeds the corpus at COMPILE
  time; a new `memory/<uid>/` enters the embed only when `src/corpus.rs`
  recompiles — local incremental builds need `touch src/corpus.rs`."
- **design.md §2/§5.2 Fix2/§5.4 + plan.toml PHASE-03 EX-1 + slice-178.toml
  `[[selector]]`** (F-5): replace `.agents/skills/close/SKILL.md` with
  `plugins/doctrine/skills/close/SKILL.md` (authored source; `.agents/` is
  gitignored). The selector fix also repairs the broken `review prime`.
- **slice-178.toml `[[selector]]`** (F-6, OPTIONAL): may add `src/corpus.rs` as a
  design-target selector alongside the F-5 fix; tolerated either way.

### Governance/spec (REV)

- None.

### Harvest → record-memory (durable platform facts, not reconcile writes)

- **F-2**: key→uid resolution via the items/ alias symlink ignores memory status
  (`hidden` filters `find` only) — retiring a capture so `<key>`→master requires
  removing the alias symlink, not just a status change.
- **F-4**: `rust-embed` + `debug-embed` embeds the corpus at compile time; adding a
  master needs `touch src/corpus.rs` for a local incremental build to see it.

## Reconciliation Outcome

All 6 findings terminal at reconcile (5 `verified`, 1 `tolerated`). No REV surface
— every write is a per-slice direct edit. No new gaps discovered (audit↔reconcile
seam intact); two corrections extended brief-adjacent for internal consistency
(user-approved).

### Direct edits applied
- **design.md D3 (§7) + §5.4 step 3** (RV-196 F-1): retire mechanism rewritten from
  the unimplementable `memory status superseded --by <master>` to the as-built
  `git rm` items/ key-alias symlink + `memory status archived`. Why stated: a
  shipped master under `memory/` can never be a supersession successor
  (`superseded --by` resolves against `items/` only). §5.3 "removed" outcome
  already correct — aligned.
- **design.md §5.4 step 3** (RV-196 F-2): added that key→uid resolution via the
  items/ alias symlink ignores memory status (`hidden` filters `find`, not
  `show <key>`), so physical symlink removal is the operative re-point step.
- **design.md §5.3 + §5.4** (RV-196 F-3): "single physical home memory/mem_019f075f…"
  and "Uid reuse … no supersede chain" corrected to the fresh uid
  `mem_019f176f71537d12b1b09826a003a602`; uid-reuse claim dropped (consistent with
  revised D3).
- **design.md §10 F2** (RV-196 F-4): inverted embed-timing note corrected —
  `debug-embed` embeds the corpus at COMPILE time; a new `memory/<uid>/` enters the
  embed only on `src/corpus.rs` recompile (local incremental needs
  `touch src/corpus.rs`).
- **design.md §2 + §5.2 Fix 2, plan.toml PHASE-03 EX-1, slice-178.toml `[[selector]]`**
  (RV-196 F-5): `.agents/skills/close/SKILL.md` → `plugins/doctrine/skills/close/SKILL.md`
  (authored source; `.agents/` is the gitignored install copy). Selector fix also
  repairs the broken `review prime` on RV-196.
- **slice-178.toml `[[selector]]`** (RV-196 F-6, optional — user-approved): added
  `src/corpus.rs` design-target selector (genuine VT embed seam, §9).

### Brief-adjacent corrections (same defect class, user-approved for consistency)
- **design.md §9 VA-2** (extends F-3): master body path `memory/mem_019f075f…/memory.md`
  → `memory/mem_019f176f…/memory.md`. Same uid-staleness as F-3; left stale it made
  the design self-contradictory and VA-2 point at a non-existent path.
- **plan.md §Phasing** (extends F-5): file-disjoint note `.agents/…` →
  `plugins/doctrine/skills/close/SKILL.md`. Same path defect in authored plan prose.

### REVs completed
- None. No governance/spec (ADR/policy/standard/spec/REQ) surface touched.

### Harvest (already durable — no new record needed)
- F-2 platform fact → `mem_019f179725c17680ab977ab7650f1707` (key→uid resolution
  ignores status). Exact match, pre-recorded.
- F-4 build-timing gotcha → covered by `mem_019e9a21f97a7d228c78013b3e8323c0`
  (RustEmbed re-embeds at compile time) + `mem_019e98a783ea7471ac4bfcefdc04ae5e`
  (re-embed footgun). No duplicate authored.

### Withdrawn / tolerated
- RV-196 F-6: `tolerated` (rationale in finding disposition); the optional selector
  add was nonetheless applied per user decision.

Reconcile pass complete — handoff to /close.
