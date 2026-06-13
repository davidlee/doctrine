# DEC prefix dual-namespaced — numbered kind (2-part) vs external the external decision register cites (3-part)

The `DEC` prefix carries two distinct namespaces, deliberately (SPEC-019 D8):

- **`DEC-NNN` (2-part)** — the numbered **decision** knowledge-record kind
  (SPEC-019, the ASM/DEC/QUE/CON family). A real entity in `integrity::KINDS`.
- **`DEC-NNN-XX` (3-part, letter/number suffix: `DEC-005-C`, `DEC-010-06`)** —
  **external the external decision register decision-log citations**, sprinkled through SL-007/012
  prose (the external decision register byte-reproduction work), `src/git.rs` comments, memory, and
  the boot snapshot. Free-text prose, **never doctrine entities, never
  renumbered** (provenance to the external decision register's own log). Doctrine's *own* doc-local
  decisions use the bare `D1` glossary form, never `DEC-`.

**Why:** before SPEC-019, `DEC` was not a kind, so the shipped `DecisionRef`
relation label (REC source) is `TargetSpec::Unvalidated` and `rec.decision_ref`
stores `DEC-005-C` as free text *by design*. Making `DEC` a numbered kind flips
that: `src/rec.rs:318`'s comment ("a DEC is … not a numbered entity kind") goes
false, and a 3-part `DEC-005-C` is **un-parseable** as a canonical id
(`parse_canonical_ref` rsplit → `"C"`, non-numeric). The collision was an
adversarial-review CRITICAL on SPEC-019.

**How to apply:** when building the DEC kind (IMP-050/051 / SL-059 family),
disambiguate the ~6 live `decision_ref` sites — the `rec.rs:318` comment, the
`relation_graph.rs`/`rec.rs` test fixtures, the `main.rs:1537` `--decision`
example — and **decide the `DecisionRef` posture**: keep it free-text
(recommended — external the external decision register refs survive) vs validate numbered DEC. Do
**not** sed-rename the 3-part external citations. See [[mem.fact.backend.the external decision register-event-store]]
and [[mem.pattern.entity.free-text-ref-not-forward-validated]].
