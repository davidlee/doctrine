# Review RV-027 — code-review of SL-060

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-close quality pass on the SL-060 source diff (`b0d3e3d^..589eb11`, `src/*.rs`;
PHASE-05 is data-only, skipped). Slice is `done`; RV-020 was reconciliation-facet
only, so the source was never code-reviewed. Findings land here, not as closure
reopening. Out of scope: the SL-062 status seam now co-resident in `src/dep_seq.rs`
(`apply_status`/`set_authored_status`/`apply_string_append`) — reviewed under RV-024.

Lines of attack:

- **Leaf strict-refuse correctness** — `append` navigates to the *array* not just the
  table; refuse is non-destructive + touches nothing; idempotency. → SOUND (unit tests
  cover missing-table/needs/after, non-destructive message, idempotent no-op).
- **INV-2 byte-identity** — backlog delegation preserves behaviour + success text.
  → SOUND (−93 LOC pure delegation; e2e asserts byte-exact messages + cycle refuse).
- **INV-3 outbound-only / F5 no-read** — → SOUND (append writes SRC only; F5 proven
  with a garbage-toml probe that is never opened).
- **D2 closed allowlist** — every non-work kind refused at author time; resolver edge
  cases. → SOUND (e2e asserts message *properties* for each refusal incl. a resolvable
  RV target; unit sweep over `integrity::KINDS`).
- **DRY / layering of the read dispatch** — does `dep_seq_for` honour the `outbound_for`
  mirror it cites? → **F-1**: it reconstructs the slice toml path inline in the engine
  tier instead of delegating to the owning module.
- **Single-parse ethos (F3)** — `slice show` now reads the slice toml twice. → **F-2** (nit).
- **Write-seam canonicalization** — raw target string stored/deduped. → **F-3** (nit),
  inherited from backlog; read canonicalizes so no resolution bug.

## Synthesis

**Overall:** solid.

**Synopsis.** SL-060 lifts the dep/seq schema + the strict edit-preserving append
into a shared `src/dep_seq.rs` leaf, generalises the priority read-gate from a
backlog-prefix test to a cross-kind `dep_seq_for` dispatch, and adds generic
`doctrine needs`/`after` verbs behind a single work-like author-time gate. The lift
is honest DRY, not parallel implementation: backlog's `append_relationship` drops to
a −93 LOC delegate, `push_str_if_absent` is shared by both append arms, and the
read divergence (backlog keeps its one-parse `read_item` for `promoted`) is the
documented, justified exception (F3). The invariants hold under inspection, not just
assertion: INV-3 outbound-only (append touches SRC only), F5 no-read (proven with a
garbage-toml probe that is never opened), D2 the closed work-like allowlist (refused
at author time with message-*property* assertions — including a resolvable RV target
to exercise the kind gate past `ensure_ref_resolves`), and INV-2 backlog byte-identity
(success-message text pinned byte-exact, cycle refuse retained in the backlog shell).
The tests are the opposite of theatre — they assert the property, name the dangler,
the self-edge, the work-only gate. The strict refuse is non-destructive and pinned
against ever instructing regeneration. `slice show` rendering is genuinely additive
(keys omitted when unauthored → byte-stable for pre-SL-060 slices).

What kept it off `solid`-without-caveat: a single minor and two nits, all cleanup,
none functional. F-1 (minor, → IMP-067): the read dispatch cites `outbound_for` as
its mirror but breaks fidelity with it — `outbound_for` delegates slice path-shape
knowledge to the slice module; `dep_seq_for` reconstructs the `slice-NNN.toml` path
inline in the engine tier, making three independent constructions of that path
(one of which, `slice::slice_toml_path`, is documented "the single chokepoint
(DRY)"). F-2 (nit, tolerated): `slice show` reads+parses the slice toml twice
against the slice's own one-parse ethos — negligible on a non-hot path. F-3 (nit,
tolerated): the write seam stores/dedups the raw target string, not a canonical ref;
no resolution bug because the read view re-parses both sides, but it carries a
cosmetic render wart + a raw-string idempotency hole, inherited from backlog and so
locked by INV-2 (the real fix is corpus-wide, not slice-local).

**Standing risks / consciously accepted:** the cross-kind author-time cycle
asymmetry (backlog refuses closing cycles pre-write; the generic slice verb defers
to read-time cordage degradation) is design-sanctioned (E3) and stated, not hidden —
a cross-kind author-time oracle is a legitimate later phase. `after` re-authored
with a changed rank appends a second edge rather than updating (dedup is exact
`(to, rank)`) — inherited from backlog, out of this review's scope.

**Haiku.**
  One leaf, two callers —
  the engine rebuilds the path
  the module should own.
