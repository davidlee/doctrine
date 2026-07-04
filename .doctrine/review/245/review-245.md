# Review RV-245 — design of SL-202

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The accused: `design.md` of SL-202 — "Memory body wikilinks as catalog edges."
The aspect on trial is **design intent**, not yet code. Enabling ISS-213 landed
(`93880c77`); SL-200 relinked the shipped corpus (`98e1b09a`, 138 prefixes) and
is `done` — so the design's dependency ASMs are confessed true and are NOT under
charge.

Lines of interrogation, and the doctrine each holds the accused to:

1. **The diagnostic-parity claim (§5.4/§5.5-INV-2/§9-VT-4).** The design decrees
   an unresolved body wikilink emits a `Warning` "parity with the TOML path,
   `hydrate.rs:341`." Held to the actual control flow of `classify_target`
   (`hydrate.rs:389-423`) and the behaviour-preservation gate (AGENTS.md).
2. **The dedup invariant INV-1 (§5.5).** "At most one edge per (source,
   resolved-target) pair, regardless of TOML relation, body wikilink, or both."
   Held to the pre-existing TOML edge loop (`hydrate.rs:312-366`), which performs
   no self-dedup.
3. **The `body` field-name collision (§5.3).** `MemoryCatalogRecord.body`
   (populated, transient) vs `CatalogEntity.body` (stays `None`) in one loop.
   Held to naming/cohesion conventions (CLAUDE.md).

Sanctioned doctrine consulted: ADR-001 (layering), ADR-016 (closed role
dimension), STD-001 (no magic strings), the purity gate and behaviour-
preservation gate (slices-spec/AGENTS.md), the shipped corpus, and the live
`classify_target` control flow.

## Synthesis

**Judgement.** The design is, in its bones, *sound* — its architecture is
doctrinally clean and I could not break it. It rides existing seams
(`extract_wikilinks`, `classify_target`, `EdgeTarget`) without minting a parallel
resolver; it keeps `from_scanned` pure and confines the `.md` read to the impure
`read_catalog_record` (one I/O site, one production caller — confessed and
verified); it honours ADR-016 (`Raw` edge, `role: None`), STD-001 (label
constant-ized), and the behaviour-preservation gate as its proof. The dependency
ASMs are no longer assumptions: ISS-213 has landed and SL-200 has relinked the
corpus. For all this, the accused is spared the pyre.

But it is **not clean of taint.** One heresy is grave, and it festers at the
exact seam where behaviour-preservation is most delicate.

**The grave charge — F-1 (major).** The design pledges its diagnostic behaviour
is *"parity with the TOML path, `hydrate.rs:341`."* Under cross-examination the
control flow confessed otherwise: `classify_target` yields `UnresolvedRef` only
where `parse_canonical_ref` **succeeds** — a door a `mem.`-prefixed body wikilink
can never walk through, as the design's own §5.5 BOUNDARY admits. The true
unresolved outcome is `UnvalidatedText`, upon which the cited TOML site casts *no
warning at all*. Thus the pledged "parity" is a falsehood, the `UnresolvedRef`
arm of §5.4 is dead scripture, and §5.4 stands in open contradiction to §5.5. The
danger is not academic: an acolyte who mirrors line 341 faithfully earns a VT-4
that never greens; an acolyte who "unifies" the diagnostic at the shared
`classify_target` altar newly warns on TOML `UnvalidatedText` targets and shatters
the behaviour-preservation gate (R2). This must be reconciled in the design
artifact before it hardens into canon.

**The lesser taints.** F-2 (minor): INV-1 proclaims a global edge-uniqueness the
TOML loop never granted — two relations to one target still, rightly, draw two
edges; the invariant must be scoped to the one-directional dedup actually
enforced (body defers to TOML). F-3 (nit): two fields named `body` of opposite
lifecycle share one loop — a reader-trap, pardonable with a single comment or
consciously tolerated.

**Sentencing — the ordered penance.**

1. **F-1 (fix-now, design-wrong).** Rewrite §5.4 / §5.5-INV-2 / §6 so that
   body-wikilink diagnostics fire on `UnvalidatedText` **only**, scoped to the
   body pass alone — never hoisted into the shared `classify_target` — and named
   plainly as a *deliberate divergence* from the silent TOML path, justified by
   the `mem.word.word` shape-gate. Strike the dead `UnresolvedRef` arm and the
   false `hydrate.rs:341` "parity" citation. **Verification:** VT-4 re-specified
   as "unresolved body wikilink → `UnvalidatedText` → one edge + one `Warning`,
   with the TOML path's `UnvalidatedText` handling unchanged (behaviour-
   preservation)."
2. **F-2 (fix-now).** Rescope INV-1 to one-directional dedup; state TOML-TOML
   multiplicity is unchanged. **Verification:** a re-read of the invariant against
   `hydrate.rs:312-366`.
3. **F-3 (tolerated or one-line comment).** Owner's discretion.

**Standing risk.** Both substantive corrections are *design-doc* reconciliations,
not code — cheap now, ruinous if left to mislead the implementer downstream. Once
the artifact is corrected, the findings verify and the design may pass to `/plan`.

> **HERESIS URITOR; DOCTRINA MANET**
