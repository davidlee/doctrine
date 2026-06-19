# Review RV-094 — design of SL-103

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Inquisition on the **design intent** of SL-103 (`.doctrine/slice/103/design.md`):
wiring the estimate/value facets into the catalog/graph hydration contract.
Codex (GPT-5.5) drives the adversarial interrogation; charges are entered here.

### Lines of interrogation

1. **D1 / OQ-1 — un-traced scope.** The design widens the slice past FR-006
   (estimate only) to expose `value` with *no governing REQ*. The design defers
   the fix to reconcile. Is deferral doctrinal, or does it ship un-traced scope?
   Is the convenience ("the generic reader carries both naturally") a real
   justification or scope-creep dressed as coherence?
2. **D2 — units contract literalism.** FR-006 names a *per-node* "project unit".
   D2 hoists units to a top-level block and asserts FR-006 is "satisfied —
   reachable via the graph". Does the contract honour the requirement's letter,
   or reinterpret it? Is reachability-through-graph an honest read of "per node"?
3. **D3 — second TOML parse.** `read_facets` re-reads + re-parses each entity
   TOML the status path already parsed. Cost dismissed as "negligible". Measured,
   or asserted? Is there a cheaper seam that does not perturb `Meta`?
4. **D4 — per-facet malformed isolation.** SL-101's facet `Deserialize`
   hard-fails ("fail loud, never repair"). D4 softens corpus-scan failure to
   diagnostic + node-without-facet. Does dropping a facet to `None` constitute a
   silent repair the design swears it avoids? Is `None`-on-malformed
   distinguishable downstream from `None`-on-absent?
5. **D5 — contract/model coupling.** Reusing `EstimateFacet`/`ValueFacet` as the
   wire contract couples the external graph contract to internal model evolution.
   Is "policy-free, stable contract" compatible with no DTO seam?
6. **Purity & behaviour-preservation.** Units resolved via disk read in the
   shell — confirm no impurity leaks into `from_scanned`. Confirm the additive
   contract evolution updates every construction site without regressing suites.
7. **Dead-code expect hygiene.** Removing the now-fulfilled `expect(dead_code)`
   vs leaving SL-102-owned symbols dead — any unfulfilled-expect clippy trap?

## Synthesis

**Judgement: HERESY FOUND. The design is NOT fit to lock.** Codex (GPT-5.5) drove
the interrogation; every load-bearing charge was cross-examined against source
before sentencing — and the cross-examination both *confirmed* the heresies and
*corrected the prosecutor*, for the accuser misnamed the cure on the gravest count.

One **blocker** stands, three **major** taints, three **minor**. The seam choices
(purity split, single `from_scanned` call site, the `parse_optional`/`resolve_unit`
signatures) were confessed sound and acquitted — the rot is in **traceability**,
in **contract honesty**, and in **error handling**, not in the mechanism.

### The corrected blocker (F-1)

The accused pleads, and the slice scope pleads, that "value graph exposure has no
requirement." Under cross-examination this premise is **true** — but the
prosecutor's proposed remedy was false witness. `SPEC-020` does carry REQ-278 /
REQ-279 / REQ-280, and codex demanded SL-103 be traced to them; yet those govern
the value **model, validation, and unit** (SL-101's flesh), *not* the exposure of
the value magnitude upon the graph. No REQ governs that exposure. Therefore the
penance is not "trace to the existing three" but **author a value-graph-exposure
REQ sibling to REQ-274 and bind SL-103 to it** (and bind SL-103 to REQ-280, whose
unit-resolution this slice actually performs) — *or* cut value from the slice.
What may **not** stand is locking and implementing un-traced scope, deferring the
requirement's very existence to reconcile. The heretic who builds before the
canon names the work builds upon sand; let the scaffold be raised before the
stone is cut. **This blocker gates the slice's close until purged.**

### The major taints

- **F-3 — false witness in the impact summary.** The design swears the map_server
  HTTP view is untouched; `src/map_server/routes.rs:156-158` serves `CatalogGraph`
  raw over `/api/graph`. The new `units` and per-node facets *will* breach the HTTP
  surface. An impact summary that omits a surface it touches is a lie of omission.
- **F-4 — `Err(_) => default` swallows the world.** §5.4 masks permission and I/O
  faults as "no config," betraying the very `coverage_store` precedent it cites,
  which defaults only on `NotFound`. A one-line correction; an unconditional sin.
- **F-2 — malformed collapses into absent.** D4 honours the letter of "never a
  repaired bound" (no coercion — acquitted on that count) but lets a corrupt facet
  wear the same wire-face as an honest absence, its corruption confessed only in a
  side-channel diagnostic. Decide the contract deliberately; do not leave it mute.

### Standing risks / lesser taints

F-5 (D2 reinterprets FR-006's per-node unit — sound reasoning, unratified), F-6
(D3's "negligible" second parse is asserted not measured, and `read_facets` erases
facets silently on a divergent second read), F-7 (D5 weds the public contract to
internal facet structs — now load-bearing because of F-3). None gate; all deserve
an explicit sentence.

### Sentence (ordered penance)

1. **F-1 (blocker):** author the value-graph-exposure REQ under SPEC-020, trace
   SL-103 → it and → REQ-280; or excise value. *Before lock.*
2. **F-4:** default only on `NotFound`, propagate the rest. One line.
3. **F-3:** correct §2/§6.3/§8; rule whether facets on `/api/graph` are in scope.
4. **F-2:** rule the malformed-vs-absent contract; document if tolerated.
5. **F-7 / F-6 / F-5:** ratify or remediate; tie F-7 to F-3's scope ruling.

Findings F-2..F-7 await the User's ruling on the substantive design questions
before disposition — they will **not** be self-disposed to greenwash the ledger.
F-1 remains an open blocker by design until the traceability is made true.