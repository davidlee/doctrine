Now I have all the data. Let me compile the comprehensive summary.

---

## Edge Validity Summary by Source Kind

Built from the single source of truth: `RELATION_RULES` in `src/relation.rs`, cross-referenced with `src/kinds.rs` for kind prefixes.

### Legend
- **Writable** — `doctrine link <SRC> <label> <TGT>` works
- **LifecycleOnly** — exists in the table but `link` refuses it; authored through a typed verb (supersede transaction)
- **TypedVerbOnly** — exists in the table but `link` refuses it; authored through a bespoke kind-specific verb
- **Target** — the legal target-kind constraint validated at `link` time (any other kind is refused)

---

### SL — Slice

| Label | Policy | Target kinds |
|---|---|---|
| `references --role implements` | Writable | SPEC, PRD, REQ |
| `references --role scoped_from` | Writable | ISS, IMP, CHR, RSK, IDE |
| `references --role concerns` | Writable | any numbered kind |
| `supersedes` | Writable | SL |
| `governed_by` | Writable | ADR, POL, STD |
| `related` | Writable | any numbered kind |

_No non-writable edges for SL._

---

### ISS / IMP / CHR / RSK / IDE — Backlog items

| Label | Policy | Target kinds |
|---|---|---|
| `references --role concerns` | Writable | any numbered kind |
| `slices` | Writable | SL |
| `governed_by` | Writable | ADR, POL, STD |
| `related` | Writable | any numbered kind |
| `drift` | Writable | free-text (Unvalidated) |

_No non-writable edges for backlog._

---

### RFC

| Label | Policy | Target kinds |
|---|---|---|
| `references --role concerns` | Writable | any numbered kind |
| `related` | Writable | any numbered kind |

_`governed_by` is NOT valid from RFC._ No non-writable edges.

---

### PRD — Product spec

| Label | Policy | Target kinds |
|---|---|---|
| `governed_by` | Writable | ADR, POL, STD |
| `consumes` | Writable | PRD |
| `parent` | TypedVerbOnly | SPEC, PRD |
| `members` | TypedVerbOnly | REQ |

---

### SPEC — Tech spec

| Label | Policy | Target kinds |
|---|---|---|
| `governed_by` | Writable | ADR, POL, STD |
| `descends_from` | TypedVerbOnly | PRD |
| `parent` | TypedVerbOnly | SPEC, PRD |
| `members` | TypedVerbOnly | REQ |
| `interactions` | TypedVerbOnly | SPEC |

---

### CM — Concept map

| Label | Policy | Target kinds |
|---|---|---|
| `contextualizes` | Writable | free-text (Unvalidated) |
| `governed_by` | Writable | ADR, POL, STD |

_No non-writable edges for CM._

---

### ADR / POL / STD — Governance

| Label | Policy | Target kinds |
|---|---|---|
| `related` | Writable | same kind (ADR→ADR, POL→POL, STD→STD) |
| `supersedes` | **LifecycleOnly** | same kind |

---

### RV — Review

| Label | Policy | Target kinds |
|---|---|---|
| `reviews` | **TypedVerbOnly** | any numbered kind |

_No `link`-writable labels for RV._

---

### REC — Reconciliation record

| Label | Policy | Target kinds |
|---|---|---|
| `owning_slice` | **TypedVerbOnly** | SL |
| `decision_ref` | **TypedVerbOnly** | free-text (Unvalidated) |

_No `link`-writable labels for REC._

---

### REV — Revision

| Label | Policy | Target kinds |
|---|---|---|
| `revises` | **TypedVerbOnly** | SPEC, PRD, REQ, ADR, POL, STD |
| `originates_from` | **TypedVerbOnly** | RFC |

_No `link`-writable labels for REV._

---

### ASM / DEC / QUE / CON — Knowledge records

| Label | Policy | Target kinds |
|---|---|---|
| `shapes` | Writable | PRD, SPEC, REQ, SL, ISS, IMP, CHR, RSK, IDE, ADR, POL, STD, RFC, ASM, DEC, QUE, CON |
| `spawns` | Writable | ISS, IMP, CHR, RSK, IDE |
| `governed_by` | Writable | ADR, POL, STD |
| `supersedes` | **LifecycleOnly** | ASM, DEC, QUE, CON |

---

### Create-invalid edges (refused by `link`, `read_block`, or `validate`)

1. **`(source, label)` off-table** — `link` says "not a relation label authorable by X via `link`" and lists the writable set. E.g. `SL → consumes`, `RFC → governed_by`, `ADR → governed_by`.

2. **Target kind mismatch** — `link` says "target must be one of [KINDS], got a <OTHER>". E.g. `SL-001 references --role implements SL-002` (implements targets SPEC/PRD/REQ only), `SL-001 governed_by SL-003` (governed_by targets ADR/POL/STD only), `ADR-001 related SPEC-001` (gov related → SameKind only).

3. **Role missing on `references`** — `link` says "requires a role".

4. **Role on a label-only label** — `link` says "does not take a role". E.g. `--role implements` with `governed_by`.

5. **Illegal role for source** — `link` says "not a legal role for X `references`". E.g. backlog item with `--role implements` (SL-only).

6. **LifecycleOnly / TypedVerbOnly via `link`** — `link` says "not `link`-writable — author it through <typed verb>".

7. **Hand-edited inverse spelling** — `read_block` → `IllegalRow(UnknownLabel)`. E.g. `superseded_by`, `governs`, `consumed_by` (derived render text, never authorable).

---

### Universal edges (every kind)

None. Every label has an explicit source-set. The widest are:

- `references --role concerns` — 7 sources (SL, RFC, ISS, IMP, CHR, RSK, IDE), target AnyNumbered
- `governed_by` — 13 sources (SL, PRD, SPEC, CM, ASM, DEC, QUE, CON, ISS, IMP, CHR, RSK, IDE), target ADR/POL/STD
- `related` — 10 sources (ADR, POL, STD via SameKind; SL, RFC, ISS, IMP, CHR, RSK, IDE via AnyNumbered)

The "universal" relations are not a single rule but the union. There is no single label every kind can author.

```
[relation.rs RELATION_RULES]
```