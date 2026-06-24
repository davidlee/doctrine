# Review RV-156 — reconciliation of SL-149

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation). **Surface reviewed:** the dispatch
candidate interaction branch `cand-149-review-001` (`candidate/149/review-001` @
`2031185c`) — a clean 3-way merge of `review/149` into `main`. `review/149` and
`phase/149-NN` are immutable evidence refs (R2); the candidate is the reviewable
surface (audit skill / ADR-012).

**Oracle.** `design.md` (D1–D6, blocker resolutions F1–F8, internal AR-1–AR-6) and
`plan.toml` per-phase EN/EX/VT. The audit holds the 6-phase rollup to:
- the structure/intent split shipped (D1/D2): `References` label + closed `Role`
  enum `{Implements, ScopedFrom, Concerns}`, target gate re-keyed
  `(source, label, role)`;
- `related` stayed a distinct symmetric-neutral label (D3); `Specs`/`Requirements`
  variants removed;
- blocker fixes landed: role-derived inbound rides as edge payload, overlay stays
  label-keyed (F1); migration role-**assignment** oracle, not edge-survival (F2);
  full in-memory transform → single atomic swap → validate-after, parser+corpus one
  commit (F3);
- the corpus migration is complete and clean (P5 EX-1..4, VT-1..3) with a committed
  disposition artifact;
- docs point at code, no transcribed vocabulary (P6, storage rule);
- behaviour-preservation gate green; integration with 19 merged trunk commits
  (SL-150 etc.) coherent.

**Evidence gathered.** On the merged tip: `just gate` clean (workspace clippy),
`just check` green incl. role-aware relation suites
(`relation_census_groups_by_label_and_role`, `relation_list_renders_role_and_label`),
`doctrine validate` reports **corpus clean** on both the coord merged tip and the
candidate. Spot checks: `References` in the enum / Specs+Requirements gone; **zero**
live `label = "specs"|"requirements"` `[[relation]]` rows; SL-149's own toml rewritten
to `references(implements) SPEC-018` + `references(concerns) RFC-003`;
`migration-dispositions.md` committed (195 edges, 136 deterministic + 59 hand-judged,
census provenance, VT-1 oracle evidence); SPEC-018 + `relation-vocabulary.md` rewritten
to references/role with no stale labels; `slice-149.md` Summary present.

## Synthesis

**Closure story.** SL-149 delivered RFC-003 Axis B as designed. The work→canon
relation family collapsed into one `references` label refined by the closed role
dimension, with target validation re-keyed from `(source, label)` to
`(source, label, role)`. All six phases are `completed`; the merged candidate builds
green, passes the role-aware suites, and `validate` reports the corpus clean. The
three external blockers (F1 role-derived inbound vs label-keyed graph; F2 migration
oracle strength; F3 "atomic" mechanism) are each resolved in the shipped code and
verified by the surviving evidence. The behaviour-preservation gate held: the
entity-engine machinery suites stayed green; only the deliberate vocabulary content
changed.

**Integration.** `refresh-base` merged 19 trunk commits (the SL-150 boot-map renderer
touching `cli.rs`/`boot.rs`/`memory.rs`, ISS-048, and edge's slice-151/issue-049/fsck)
**cleanly, no conflicts**. The migration snapshot (taken at `ccc42c5`, trunk +9) is
behind the final merged corpus, but the delta is memory-space `related`/`drift` edges —
labels SL-149 deliberately **keeps** (D3) — so they validate without re-migration.
Integration is coherent.

**Consciously accepted tradeoffs / standing risks.**
- The **hybrid execution deviation** (F-2): P2–P4 shipped additively and the hard cut
  moved to P5. This is the correct shape, not drift — the migration must read the old
  edges before they can be removed, so co-locating the cut with the migration is where
  design §2.9 always put it. User-approved via /consult, durable in `notes.md`.
- **Migration is a point-in-time snapshot.** New `references` edges author with the
  grammar going forward (P6 docs teach it); the migration does not retro-police edges
  added after its census. Accepted by design (AR-1, residual caveat).
- **`scoped_from`/`part_of` boundary** held by definition + target-kind gate, not a
  structural invariant (F7, accepted residual) — cleanup deferred to Axis D `part_of`.

No blockers. The two non-trivial gaps (web-graph TS rendering; SPEC cross-references)
are genuine future work, routed below.

## Reconciliation Brief

Handoff to `/reconcile`. No per-slice design/governance edits are required — the
design and the shipped code agree (the deviations are documented and intended).
Every item below is **owned future work** surfaced by the audit; the slice itself is
conformant.

### Per-slice (direct edit)
- None. `design.md` already describes the shipped contract (the hybrid deviation is
  recorded in `notes.md`, not a design correction). No prose drift to repair.

### Governance/spec (REV)
- None required for closure. SPEC-018 + `relation-vocabulary.md` were rewritten in P6
  and are coherent with the code.

### Follow-up work (backlog new)
- **F-3 (major) — web-graph TS frontend role rendering.** Backend
  (`catalog/graph.rs`) serialises `edge.role`; the TS frontend (`web/map/`) does not
  yet read it to render `references(<role>)` in the dot label. File an improvement.
- **F-4 (minor) — rewire SPEC-005/006/016 to reference SPEC-018.** Now that the
  relation contract lives in code + SPEC-018, the specs that describe relation
  vocabulary should reference it rather than re-transcribe. File a chore/improvement
  (or fold into an existing axis follow-up).
- **F-5 (nit, tolerated) — stale `specs` label example** in
  `mem_019ec0d9.../memory.md`. Optional memory/dreaming hygiene; not worth a dedicated
  backlog item unless dreaming sweeps it.

## Reconciliation Outcome

Reconcile pass complete. **No write-surface changes** — the design and the shipped
code agree (the hybrid execution deviation is documented in `notes.md`, not a design
correction), and P6 already brought SPEC-018 + `relation-vocabulary.md` coherent with
the code. No per-slice direct edits and no REVs were required.

### Direct edits applied
- None.

### REVs completed
- None.

### Follow-up work filed (audit-surfaced, owned future work)
- **IMP-168** — Web-graph TS frontend renders `references(<role>)` in the edge label
  (covers RV-156 F-3).
- **CHR-026** — Rewire SPEC-005/006/016 to reference SPEC-018 (covers RV-156 F-4).

### Knowledge harvested
- `mem.pattern.jail.stale-test-fixture-vocabulary-change`
  (`mem_019ef8c35b407a738b66e1fa5eaaa0f3`) — the stale-test-binary-embeds-old-fixture
  footgun + the `| tail` exit-code mask that hid it during P05.

### Withdrawn / tolerated
- RV-156 F-5: tolerated — stale `specs` example in a foreign-owned memory body;
  rationale in the finding disposition. Optional dreaming sweep.

Reconcile pass complete — handoff to /close.
