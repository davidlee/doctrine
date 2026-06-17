# Review RV-059 — design of SL-090

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisition arraigns the design of SL-090 ("Wire link/unlink CLI for memory
relations") on six lines of interrogation:

1. **Scope-design alignment** — does `design.md` contradict the approved
   `slice-090.md`, especially the shipped/ fallback that D4 appears to tighten
   to items/-only?
2. **Coherence with existing machinery** — do the proposed `resolve_memory_toml_path`,
   `append_memory_relation`, and `remove_memory_relation` compose cleanly with the
   existing `relation.rs` write seam (`append_edge`/`remove_edge`/
   `RELATION_RULES`/`AppendOutcome`/`RemoveOutcome`), or do they fork a parallel
   write path?
3. **ADR-001 layering** — does D5's reuse of `AppendOutcome`/`RemoveOutcome`
   respect the `leaf ← engine ← command` module arc?
4. **D3 target validation semantics** — does "best-effort" canonical-ref
   validation compose with the free-text/memory-UID pass-through without an
   ambiguous edge case?
5. **D4 shipped/ semantics** — is "clone to items/ first" a real, actionable
   remedy or a dead end?
6. **D6 F1 trap defence** — does the design specify *how* the guard is reused
   (public vs duplicate), or does it hand-wave?

The Inquisition holds the design to: the approved scope, ADR-010 (relation
modelling), ADR-004 (outbound-only), ADR-001 (module layering), and the
project convention that design decisions must be decidable — an implementor
must not arrive at an ambiguous fork.

**Presumption: guilty until proven clean.**

## Synthesis — the verdict

The design is **guilty on one count of heresy against the approved scope** (F-1,
blocker) — D4 silently contradicted `slice-090.md`'s "items/ first, shipped/
fallback" with an "items/ only" gate — and **guilty on three counts of
indecision** (F-2/F-4/F-6 — major) that would force the implementor to stop and
design at the keyboard. Four lesser taints (F-3/F-5/F-7/F-8) round out the ledger.

**The heresy is corrigible without excommunication. The penance is prescribed.**

### Ordered corrective penance

1. **F-1 (blocker) — Reconcile D4 with the scope.** Either reinstate shipped/
   fallback (items/ write, shipped/ read-only) per the scope, or file a scope
   amendment recording the narrowing with rationale. The scope is the governing
   authority until amended.

2. **F-2 (design-wrong) — Fix the shipped/ error path.** If shipped/ fallback is
   restored (F-1), unlink on a shipped/-only uid works read-only. Otherwise,
   replace "clone to items/ first" with an honest message that names the actual
   available remedies (hand-add to items/, backlink a new record).

3. **F-4 (major) — Resolve D6's fork.** The right path: duplicate the F1 guard
   and append/remove logic in `memory.rs`, adapted for free-form label strings.
   `RelationLabel` is vocabulary-bound; memory labels are raw per D2. Sharing
   the toml_edit pattern and guard shape, not the enum, is correct.

4. **F-3 (minor) — Document the import direction.** Add to D5: "Import direction:
   `memory.rs` imports `relation::AppendOutcome`/`RemoveOutcome` (leaf → leaf,
   no cycle)."

5. **F-5 (minor) — Disambiguate test table.** Add the word "SOURCE": "Nonexistent
   SOURCE uid (items/) → error."

6. **F-6 (minor) — Specify the path joiner.** Add to the `resolve_memory_toml_path`
   description: "uses `fsutil::safe_join` (the H1 chokepoint), same as
   `resolve_show`."

7. **F-7 (nit) — Add help-text update to code-impact table.**

8. **F-8 (minor) — Update handover after corrections land.**

### Standing risks

- **The `RelationLabel` vocabulary remains closed to memory labels.** The
  design's D2 (raw labels, no `RELATION_RULES` gate) is correct — the catalog
  already handles `CatalogEdgeLabel::Raw`. But it means the memory write path
  forks from the numbered-entity write path at the label type. Any future
  "unified label vocabulary" work will need to reconcile these.
- **Shipped/ fallback for unlink creates an asymmetry** — link always needs
  items/ (writable), unlink on shipped/ is read-only removal of a row that may
  be hand-authored or system-generated. The shipped/ regeneration (`doctrine
  memory sync`) will re-materialize the relation from the binary, undoing the
  unlink. This is a data-model tension, not a design defect — but it should be
  noted.

### Tradeoffs consciously accepted

- **Parallel write path.** Memory relations use a free-form label write path
  beside the `RELATION_RULES`-gated `link`/`unlink` path. This is the cost of
  D2 (raw labels) and is already priced into the catalog read path
  (`CatalogEdgeLabel::Raw`).
- **No `RelationLabel` variant for memory edges.** Adding one would couple the
  vocabulary to user-chosen label strings — a worse outcome than the fork.

The Inquisition finds the design **salvageable** — the corrections are small,
well-scoped, and do not disturb the architecture. But they MUST land before
/plan proceeds. An implementor who reads the design as-is will hit F-1 and stop,
or worse, drive past it into F-4's ambiguous fork.

**HERESIS URITOR; DOCTRINA MANET**
