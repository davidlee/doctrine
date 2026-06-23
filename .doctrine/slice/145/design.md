# Design SL-145: Backlog relation source parity

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

RFC-003 Axis A: BACKLOG kinds (ISS/IMP/CHR/RSK/IDE) cannot author the reference
relations they legitimately need. CHR-024 findings F-1/F-2/F-4/F-5 reduce to one cause:
the `RELATION_RULES` `sources` sets for `governed_by` and `related` exclude BACKLOG, so a
chore "governed by ADR-010" or an improvement "related to SL-099" has no structured
home and is forced into prose. Make those two edges legal for backlog sources, end to
end through the CLI — nothing more.

## 2. Current State

`RELATION_RULES` (`src/relation.rs`) is the single legal-set vocabulary table keyed by
`(source ∈ sources, label)` (ADR-010 D2). Today:

- `governed_by` — `sources: [SL, PRD, SPEC, CM, ASM, DEC, QUE, CON]`, `target:
  Kinds(GOV)`, tier-1, Writable. **BACKLOG absent.**
- `related` — two rows: `GOV` (`SameKind`) and `[SL, RFC]` (`AnyNumbered`), both tier-1
  Writable. **BACKLOG absent.**

Read path (the reason this is a table-only change): `backlog::relation_edges`
(`src/backlog.rs:935`) returns `item.tier1` — **every** legal `[[relation]]` row read
generically in canonical order. It is not per-label. `read_block` (`src/relation.rs`)
drops rows whose `(source, label)` is illegal (`IllegalReason::IllegalForSource`) — which
is exactly why a hand-authored backlog `governed_by` row vanishes on read today. The
write verb (`doctrine link`) routes through `validate_link`, which refuses the same
illegal pair up front. So the entire author→persist→read→inspect loop is already generic;
only the legality gate stops backlog.

Tests pin the table:
- VT-2 `sources_match_shipped_accessors` (`relation.rs:1076`) restates each label's legal
  source set; `Related` is currently `[ADR, POL, RFC, SL, STD]`. **`GovernedBy` is not in
  VT-2's `expected` list** (it carries no typed accessor emit), so widening `governed_by`
  does not touch VT-2.
- `read_block` golden (`relation.rs:1429`) asserts a backlog `governed_by` row is
  `IllegalForSource`.

## 3. Forces & Constraints

- **ADR-004** (relations outbound-only; reciprocity derived) — unchanged: backlog authors
  the outbound edge, the inbound (`governs` / `related`) is derived on the target. No new
  storage of reverse edges.
- **ADR-010** (unify contract + write seam, keep storage bespoke) — `sources` is a SET so
  one rule serves many kinds; widening a set is the sanctioned extension shape (no new row
  per kind).
- **Behaviour-preservation gate** — widening a `sources` set adds legality; it removes
  none. Every existing edge reads/validates exactly as before. The only test churn is the
  goldens that *asserted the refusal*.
- **SPEC-018** — the cross-corpus relation contract; this slice widens two cells, asserts
  no new contract.
- **RFC-003 Layer 1** — graph-effect (gating/eviction/scoring) is a consumer decision, not
  a relation property. This slice touches the vocabulary only; no consumer reacts.

## 4. Guiding Principles

- Smallest legal-set widening that closes Axis A. No new label, no role dimension, no
  migration, no consumer change.
- Ride the generic read path; do not add a backlog-specific accessor branch.
- The table is the contract; the tests are the proof. Flip exactly the goldens that
  encoded the old refusal, no more.

## 5. Proposed Design

### 5.1 System Model

Two table edits, two test edits, all in `src/relation.rs`:

1. `governed_by` row — `sources` becomes `[SL, PRD, SPEC, CM, ASM, DEC, QUE, CON] +
   BACKLOG`. Target gate (`Kinds(GOV)`) unchanged: backlog→ADR/POL/STD only.
2. `related` `[SL, RFC]` row — `sources` becomes `[SL, RFC] + BACKLOG` (D1: extend, not a
   new row). Target stays `AnyNumbered`; backlog may relate to any numbered entity.

No change to `backlog::relation_edges`, `validate_link`, `read_block`, `inspect`, or any
surface — they consult the table and now admit the widened pairs.

### 5.2 Interfaces & Contracts

CLI surface (already generic, now legal for backlog):
- `doctrine link <BACKLOG-id> governed_by <ADR|POL|STD>` — authors a tier-1 `[[relation]]`
  row in the item's TOML; `validate_link` accepts.
- `doctrine link <BACKLOG-id> related <any-numbered>` — same.
- `doctrine unlink …` — removes the row (generic, round-trips).
- `doctrine inspect <BACKLOG-id>` — outbound renders `governed_by`/`related` (the
  authoritative relation render).
- `doctrine inspect <target>` — derived inbound renders `governs` / `related`.
- `doctrine backlog show <id>` — relations surface via the same `item.tier1`; the `show`
  summary may render a label subset (cf. `slice show`, which summarises only some labels),
  so `inspect` is the verification oracle, `show` a best-effort confirm.

Write seam (verified kind-generic, `src/commands/relation.rs`): `run_link` →
`resolve_link_path` resolves the source `KindRef`, calls `validate_link(kind, label)`,
and `append_edge` writes the row to `id_path(root, kind, id, Toml)`. **No command-layer
kind allowlist** — `validate_link` (the gate this slice widens) is the sole legality
check. `run_unlink` is symmetric and does no target validation. So widening the table is
sufficient for the full CLI loop; no command code changes.

Target-kind gate is unchanged and still enforced: `link CHR-024 governed_by SL-099` is
still refused (`SL` ∉ `Kinds(GOV)`) — only the *source* widened, not the target.

### 5.3 Data, State & Ownership

Storage shape unchanged: tier-1 `[[relation]]` rows in the backlog item's `*.toml`,
authored only via `link` (the relate-via-link rule, [[pattern.relation.relate-via-link-not-hand-authored-rows]]).
No migration — this permits new edges; it rewrites no stored row. Existing backlog
`slices`/`specs`/`drift` rows are untouched.

### 5.4 Lifecycle, Operations & Dynamics

No lifecycle interaction. The edge is durable structural intent; it does not move with
status. (Contrast the `slices` temporal reading deferred in RFC-003 — not in scope here.)

### 5.5 Invariants, Assumptions & Edge Cases

- **INV:** widening `sources` is monotonic — the legal pair set strictly grows; no
  previously-legal pair changes target/tier/inbound. Proven by every non-flipped test
  staying green unchanged.
- **Edge — duplicate label rows:** `related` already legally appears on multiple rows
  (GOV + [SL,RFC]); adding BACKLOG to the second keeps lookup unambiguous (disjoint source
  sets per row). No `(source,label)` collision.
- **Edge — target gate intact:** backlog `governed_by` still validates target ∈ GOV;
  backlog `related` accepts AnyNumbered. Dangling/illegal-target findings behave as for
  SL today.
- **Assumption:** no overlay/accessor assumes BACKLOG is absent from these labels.
  Verified by reading `backlog::relation_edges` (generic `item.tier1`) and confirmed by
  the full suite staying green.

## 6. Open Questions & Unknowns

All three scope OQs are resolved by the §7 decisions:
- OQ-1 (review outlet, F-5) → D2 defer to Axis B.
- OQ-2 (`related` row strategy) → D1 extend existing row.
- OQ-3 (consumer reaction) → D3 permit-only, no consumer change.

No residual unknowns. (Inbound-render naming is already pinned: `related` inbound =
"related"; `governed_by` inbound = "governs" — both pre-existing, VT-3 covered.)

## 7. Decisions, Rationale & Alternatives

- **D1 — extend the existing `related` `[SL, RFC]` row to include BACKLOG** (vs a separate
  BACKLOG row). Backlog wants the identical shape SL/RFC have (AnyNumbered, Writable,
  inbound "related"); a second row duplicates that shape for no semantic gain and grows
  the lookup fan. VT-1 enum-order is unaffected either way (no new label variant).
- **D2 — defer the review outlet to Axis B.** `reviews` is `RV`-only `TypedVerbOnly`; a
  non-RV backlog reviewer edge has no clean home without B's role grammar, which reserves
  `references(reviews)` for precisely this. Minting an interim label here would be
  collapsed by B — churn for nothing. F-5 stays open against B.
- **D3 — permit the edge only; no consumer reaction.** RFC-003 Layer 1: graph-effect is a
  consumer decision. Priority overlay / `/close` / transitive-walk changes are out of
  scope. Permitting the structured edge is the entire deliverable.

## 8. Risks & Mitigations

- **R1 — a hidden consumer assumes backlog never carries `governed_by`/`related`** (e.g. a
  match that panics on an unexpected label for a backlog kind). *Mitigation:* both read
  (`item.tier1`, generic) and write (`run_link`→`append_edge`, no kind allowlist) paths
  are label-agnostic and route through `validate_link`; full-suite green is the proof.
  Grep for backlog-kind-specific label matches during execute.
- **R2 — VT-2 expected drifts silently** if `Related`'s set is updated wrong.
  *Mitigation:* VT-2 compares the table against the hand-listed expected; an exact-set
  assertion fails loudly on any mismatch. Update once, deliberately.
- **R3 — installed jail binary is stale** ([[pattern.relation.authored-rows-tooling-half-wired]]):
  CLI end-to-end checks must run against a fresh `./target/debug/doctrine`, not the RO
  jail binary.

## 9. Quality Engineering & Validation

Red/green/refactor, behaviour-preservation gate held:

- **Unit (flip the refusal goldens):**
  - `read_block` golden (`relation.rs:1429`): backlog `governed_by ADR-010` now emits an
    edge; `illegal` is empty for that row. Extend with a backlog `related <numbered>`
    legal-emit assertion.
  - VT-2 `sources_match_shipped_accessors`: `Related` expected →
    `[ADR, POL, RFC, SL, STD, ISS, IMP, CHR, RSK, IDE]`.
  - Add a positive `validate_link` assertion: `(ISSUE_KIND, "governed_by")` and
    `(ISSUE_KIND, "related")` resolve to their rules (mirror the slice `governed_by`/
    `related` legality tests).
  - Add a target-gate negative for a backlog source: backlog `governed_by` → non-GOV
    target still refused (gate intact).
- **Behaviour-preservation:** the rest of the relation suite, `relation_graph`, integrity,
  and inspect goldens stay green **unchanged**. Any unexpected churn is a design defect.
- **CLI end-to-end (manual, fresh dev build):** `link` a real `CHR`/`IMP` item
  `governed_by` an ADR and `related` to an entity; confirm the `[[relation]]` row lands in
  the item TOML, `backlog show` + `inspect` render outbound, the target's `inspect` renders
  the derived inbound, and `unlink` round-trips.
- **Gate:** `just gate` clean (clippy zero-warning, fmt).

## 10. Review Notes

Internal adversarial pass (pre-plan):

- **Claim "BACKLOG absent from `related`" challenged.** The test comment at
  `relation.rs:1681` reads "new BACKLOG/SLICE row" for SL-095. **Resolved:** loose
  historical phrasing — the actual row is `sources: &[SL, RFC]` (no backlog); code is
  authoritative. The comment becomes genuinely accurate after D1. *Action:* optionally
  tidy the comment during execute (non-blocking).
- **Write-path allowlist risk (the user's "CLI end-to-end" concern).** Probed
  `src/commands/relation.rs`: `run_link`/`run_unlink` are kind-generic, no source-kind
  allowlist, `validate_link` is the sole gate. **Resolved:** table widening is sufficient
  for the full loop. Folded into §5.2 and R1.
- **VT-2 prose comment accuracy.** The `sources_match_shipped_accessors` doc-comment
  (relation.rs:1054–1060) states "backlog::relation_edges emits slices/specs/drift" — after
  widening it also emits `governed_by`/`related` (generically). *Action:* update that
  comment alongside the `Related` expected-set edit so the prose doesn't lie. Tracked in
  the plan.
- **`backlog show` render coverage.** `show` summaries are label-selective (observed on
  `slice show`). **Resolved:** `inspect` is the verification oracle; §5.2/§9 adjusted.
- **No residual design uncertainty** — proceeding to the user's review choice.
