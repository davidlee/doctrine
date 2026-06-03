# slice-002 handover — adversarial design review + adjudication

slice-002 (kind-parameterised entity engine) was reviewed adversarially by a
fresh agent before any build. The review reshaped the roadmap: the engine
extraction is folded into **slice-003** (extracted against a real second caller —
the design-doc sibling), and slice-002 is superseded. This file is the audit
trail: the review verbatim, then how each finding was dispositioned.

The review targeted the whole entity model (slices-spec, reservation-spec,
relation-index, drift-spec, spec-entity-spec, slice-002), not just slice-002.

---

## Review verbatim

**Verdict: Amber, leaning red.** Thesis (lift relational data out of prose docs,
join by stable id, derive what's derivable, defer integrity to a registry) is
correct and the spec-driver diagnosis is accurate, not over-fit. But three
load-bearing *execution* claims are false or unguarded as written.

### FATAL

- **F1 — "git-ref composes without changing callers" is falsified by slice-002's
  own non-goals.** reservation-spec says the caller never changes; slice-002 says
  it won't add `LeaseBackend`. But `reserve_create` is built on `fs::create_dir` +
  `ErrorKind::AlreadyExists`, a different shape/signal/linearization point than
  `acquire() -> Won | AlreadyHeld`. Retrofitting = a caller rewrite. The
  unification is *nominal* until the seam exists in code. **Fix:** extract against
  a one-method `acquire` seam now (local `mkdir` the only impl). *Secondary:*
  git-ref reserve does a network round-trip (`fetch --prune`); local is offline —
  "only the reach changes" is too strong.

### MAJOR

- **M1 — Sub-entity ids (requirement local-id, drift `ref`) bypass the
  reservation primitive — the exact hazard it prevents.** Hand/sequence-assigned,
  concurrent adds + clean merges produce silent duplicate ids that pass a
  write-time uniqueness check. **Fix:** hard duplicate-id lint at *load over
  merged state*; make collisions loud, not silent (can't `mkdir`-arbitrate a row).
- **M2 — The spec decomposition reintroduces toml-row/md-heading duplication and,
  unlike drift, specifies no atomic writer and no lint.** The hairiest entity gets
  the weakest drift protection. **Fix:** spec inherits drift's atomic add +
  orphan lint per table.
- **M3 — The engine's abstraction boundary is one notch too high: the *fileset* is
  kind-specific.** slice/drift = 2 files, spec = 8 (really ~13). `Scaffold`
  hardcodes the pair; the engine reaches drift but not spec → fork risk. **Fix:**
  fileset is a kind-supplied function, not a fixed toml+md pair.
- **M4 — `collaborators[]` has no home in the decomposition; validated against a
  spec where it's empty.** Cross-spec, requirement-level edge — not derivable, not
  an interaction. **Fix:** add a `[[collaborator]]` table.
- **M5 — Closed `DriftKind` on detector-emitted data is inconsistent with the
  open-`observed` rationale, and one unknown kind fails the whole ledger.**
  **Fix:** `Other(String)` → warned row, not dead file.
- **M6 — The headline value (referential integrity) is entirely deferred, with no
  trigger or owner; v1 is no better than spec-driver.** **Fix:** name the registry
  as the unblock-gate with a falsifiable trigger (first cross-spec FK), not "feels
  slow."

### MINOR

- **m1** — `status=audit` (slice stage) vs `AUD-` (audit entity) collide on the
  word. → cross-reference.
- **m2** — the design discards single-file *read*-locality, not just find. → name
  the cost.
- **m3** — relation-index's "few thousand docs" undercounts ~8× (files). → restate
  the threshold in files.
- **m4** — four of five slice statuses are never set by tooling. → note the seam is
  fully manual.

### Steelman

Don't build slice-002 yet. One real caller (`slice.rs` works); the two justifying
callers are registry-gated with no trigger; and the abstraction is provably
insufficient for one of them (M3). Extract on the *second concrete use*, not in
anticipation. If it must proceed, do F1's `acquire` seam now; defer the rest.

### Unverified (author questions)

1. Is `collaborators[]` ever non-empty in the corpus? 2. Do requirements move
between specs (compound key then not a stable global address)? 3. Who writes the
drift ledger in v1 (raises/lowers M5)? 4. Is there a planned registry trigger, or
"build when slow" (sets M6 severity)?

---

## Adjudication (author)

Verified against the live schema bundle (`spec-driver-schemas.local.md`,
gitignored): `primary[]` = the ids in `requirements.toml` (derive holds);
`collaborators[]` is a real, fully-qualified cross-spec requirement FK (M4
confirmed — SPEC-110's empty value hid it); spec carries **7** blocks, not 4.

| Finding | Call | Landed in |
|---|---|---|
| F1 | Accept (reframe: spec consistent, code shortcut) | reservation-spec § Code seam; slice-003 scope (acquire seam first) |
| M1 | Accept | drift-spec § Known risks + Testing (load-over-merged hard lint); spec-entity-spec § Known risks |
| M2 | Accept fix, **reject framing** (row+prose join is the intended pattern, not duplicate fact) | spec-entity-spec § Known risks (orphan lint + atomic add) |
| M3 | Accept (best finding) | slice-003 (fileset-as-function + optional reservation); spec-entity-spec § Follow-ups |
| M4 | Accept (confirmed) | spec-entity-spec decomposition + Mapping + `collaborators.toml` |
| M5 | Accept | drift-spec § Serde types (`Other(String)`) + risks/testing |
| M6 | Accept fix, **reject "no better than spec-driver"** (structure/diffs/queryability land now) | relation-index § Two purposes (FK-validation trigger = first cross-spec FK); spec-entity-spec § Known risks |
| m1–m4 | Accept all | slices-spec § Lifecycle (m1, m4); spec-entity-spec § Known risks (m2); relation-index § count-in-files (m3) |
| Steelman | **Agree** | slice-002 superseded; engine folds into slice-003, extracted against slice + design-doc callers |

Open author-questions 2 and 3 are recorded as live open questions
(spec-entity-spec § Open questions 3; drift-spec § Detection).

---

# Round 2 — adversarial review + adjudication

A second fresh agent reviewed the entity model (focus: the spec entity). Unlike
round 1, this review verified its claims against the live schema bundle, so most
findings are real. It is a **notes-tightening** pass — every blocker resolves to
a pinned decision or a corrected field, not a redesign — and it does **not**
touch slice-003 (the engine + design-doc build), `src/slice.rs`, or the `acquire`
seam, so nothing here blocks the active slice. No code changed.

## Review verbatim

**Verdict.** Direction sound, would not land as-is. Diagnosis right (embedded YAML
blocks are relational data hidden in prose → parse-heavy queries, duplicated
requirement lists, weak FK integrity, merge-hostile prose-in-data). Split into
table TOML + prose siblings attacks the right disease. Problems are in the
boundary conditions: canonical ids, schema compatibility, row/prose drift, and
when validation becomes mandatory.

**Blockers.**
1. **Spec directory identity under-specified, probably collides.** `PRD/SPEC/REV`
   share one shape but the example is `.doctrine/spec/110/`, `id=110`,
   `kind="spec"` — only safe if numeric ids are globally unique across the three;
   else `PRD-110`/`SPEC-110`/`REV-110` collide on disk. Fix: kind-scoped dirs
   (`.doctrine/spec/SPEC/110/` or `.doctrine/spec/spec-110/`) and reservation
   namespace `<kind>/id/<n>`. The reservation layer is already generic enough.
2. **Requirement reference format inconsistent.** `coverage.toml` uses local
   `requirement="FR-002"`; collaborators use qualified `SPEC-200.FR-010`.
   spec-driver stores `primary` fully-qualified and `coverage.requirement` is
   schema-patterned fully-qualified. Pick a rule: rows use local ids only for
   owned rows; every cross-table FK stored fully-qualified, even within the
   owning spec; CLI may render local shorthand.
3. **Row/prose split breaks inherited schema unless Heresiarch defines a new
   one.** `spec.requirements@v1` requires `description` and `acceptance_criteria`;
   the mapping lifts both to prose — not lossless unless Heresiarch explicitly
   replaces the schema. Fix: keep query-relevant fields (incl. acceptance) in
   TOML, let prose expand them; don't lose acceptance-criteria queryability.
4. **FK validation can't stay "later" once cross-spec refs exist.**
   `collaborators.toml`/`interactions.toml` are cross-spec, so `heresy validate`
   is part of minimum viable spec v1 (`spec new` · `req add` · `show` ·
   `validate`). Without it the decomposition pays the file-count/read-locality
   cost while keeping the dangling-FK failure.
5. **Serde sketches too stringly; one field wrong.** Use raw-parse vs internal
   model (newtyped FKs, parsed enums). `Interaction.description: String` is
   required but the TOML uses `notes`, and the schema makes both `notes` and
   `description` optional (only `type`+`spec` required).
6. **Requirement moves need a decision before external refs accumulate.**
   `SPEC-110.FR-001` isn't stable if requirements move. Decide now — Option A
   (never move; move = retire+reintroduce+supersedes) or Option B (immutable UID
   + display key). Recommends A for auditability.

**Non-blocking.** Add a locality recovery command (`heresy spec req show
SPEC-110.FR-001`) rendering row+prose+coverage+caps+refs together. Use
`toml_edit` for mutations if comments/formatting/unknown-keys matter (plain
serde reserialize drops them; slices-spec promises preservation). Capabilities:
keep as a table only if stably-id'd and referenced, else collapse to tags later
(already open).

## Verification (author, against the schema bundle)

- **§4 artefact map:** `prod`→`product/PROD-xxx/`, `spec`→`specs/tech/SPEC-xxx/`,
  `revision`→`revisions/RE-xxx.md` — three kinds, three prefixes, three trees,
  independently numbered (PROD-011 and SPEC-110 coexist). **B1 confirmed real.**
- **`verification.coverage.requirement`** pattern `^(SPEC|PROD|ISSUE)-…\.(FR|NF|
  NFR)-…$` — mandatorily fully-qualified. Note's local `FR-002` violated it.
  **B2 confirmed**; `primary[]` example is FQ too, but the derive rule survives
  (own rows, qualified) — no conflict with round-1's "derive, don't store."
- **`spec.requirements`** item `required:[id,title,lifecycle,kind,description,
  acceptance_criteria]`. **B3 half-real:** the fields are required, but
  Heresiarch deliberately defines its *own* shape (it abandons the embedded
  block), so "breaks the schema" over-claims. The substance — acceptance is a
  testable list, not narrative — is right.
- **`spec.relationships`** interaction `required:[type,spec]`; `notes`/
  `description` optional; `type` is **free-text** (no enum). **B5 confirmed** —
  and the note's own `InteractionType` closed-enum sketch contradicted its own
  "free-text type" comment; both fixed to `String`.

## Adjudication

| Finding | Call | Reason | Landed in |
|---|---|---|---|
| B1 spec dir identity | **Accept (mod)** | Real per §4 — three independently-numbered kinds, not one space. Reframed: not a bare collision but kind-scoping → three engine descriptors (own dir/numbering/namespace `<kind>/id/<n>`). | spec-entity § Spec identity (new); § The decomposition; reservation-spec § Key table |
| B2 ref format | **Accept** | `coverage.requirement` is schema-FQ; note's local form was wrong. Rule: bare id only as own row id, all FKs qualified. Derive-primary survives (rendered FQ) — no round-1 conflict. | spec-entity Three Rules r4; coverage example; Serde comments |
| B3 schema break | **Accept fix, reject framing** | Keep `acceptance_criteria`/`success_criteria` as structured arrays (testable, queryable, per-row split already isolates merges); `description`/`summary`→prose. Reject "must replace schema / lossless translation": Heresiarch reshapes by design; fidelity is to data, not the block schema. | Diagnosis bullet 4; Mapping; requirements example; Serde `Requirement` |
| B4 validate co-lands | **Accept (mod)** | Sharpening of the existing trigger, not new: the cross-spec tables *are* the trigger, so validate ships in the spec slice's minimum bundle, not after. Cheap + cache-independent. (No "v1 = spec-driver" claim made; round-1 M6 reject still holds.) | spec-entity § Known risks (integrity); relation-index § Two purposes |
| B5 stringly + field | **Accept bug; defer layering** | `Interaction.description` required→optional, add `notes`, `type`→`String` (free-text per schema; fixes note-internal contradiction). Raw/model two-layer = build-time, noted not pinned in a deferred note. | spec-entity § Serde types |
| B6 requirement moves | **Accept** | Not re-litigation (round-1 left it open, not rejected). Adopt Option A now — cheap policy, makes the compound key a permanent address, avoids early hidden UID and the expensive retrofit. | spec-entity § Known risks; § Open questions 3 (narrowed to refactor mechanics) |
| NB locality CLI | **Accept** | `heresy spec/req show` reassembles the split at read time — the read-locality mitigation. `spec show` already in the B4 minimum bundle. | spec-entity § Follow-ups; read-locality risk |
| NB toml_edit | **Accept** | Real: mutating verbs that serde-reserialize drop comments + unknown keys the notes promise to preserve; edit-preserving append required. | spec-entity orphans risk; drift-spec self-drift risk |
| NB capabilities | **Reject (no-op)** | Already Open question 1; review concurs it's open. No change. | — |

## New finding (surfaced by verification, not in the review)

`PRD`/`REV` do **not** share `SPEC`'s fileset: §4 shows `prod` carries *no* fenced
blocks (frontmatter + prose) and `revision` is a single `revision.change` file —
so "the spec family shares one internal shape" holds for the row+prose
*discipline* but not the fileset. The fileset-as-function engine (slice-003)
already admits this; pinning each kind's fileset is recorded as spec-entity
§ Open questions 4. No action now (deferred), but it strengthens B1's
three-descriptor framing.

## Open questions for the user

1. **Spec-family directory layout.** B1's fix pins *kind-scoping*; the exact form
   is left as `.doctrine/{prd,spec,rev}/<n>/` (sibling trees, matches the engine's
   per-kind `dir`). spec-driver instead nests (`specs/tech/SPEC-xxx/`). Sibling
   trees chosen for descriptor-simplicity; flag if a nested `.doctrine/spec/<kind>/`
   is preferred.
2. **PRD/REV filesets** (§ Open questions 4) — pinned when those kinds are
   designed; no blocker now.

---

# Round 2 addendum — drift ledger schema reconciliation

After the round-2 disposition, the user added the **canonical drift-ledger
schema** to the bundle (§ 3c — `DriftEntry` / `DriftLedger` pydantic models),
which had been missing. drift-spec could finally be *verified* rather than
modelled on memory — and it diverged materially, because the round-1 note was
built on the **legacy minimal sweep variant** (`target` / `drift_kind` /
`disposition` / `detail`, § 3c "Observed minimal variant"), not the canonical
model. That variant only parses because of progressive strictness (unknown keys
→ `extra`); it is not the first-class shape.

Verified gaps and the realignment (all in `doc/drift-spec.md`):

| Was (round-1, legacy-variant) | Canonical model | Fix |
|---|---|---|
| `kinds: Vec<DriftKind>` (6 invented, **array**) | `entry_type` — 5 values, **singular** | `entry_type: EntryType` |
| `disposition` (amend/accept/defer/dismiss) | **no such field** | split → `assessment` (confirmed/disputed/deferred/not_drift) + `resolution_path` (ADR/DE/RE/backlog/editorial/no_change) |
| `EntryStatus` — 4 values | 7 values | + `triaged`, `adjudicated`, `superseded` |
| (none) | `severity` | `Severity` (blocking/significant/cosmetic) |
| `observed: BTreeMap` (open map) | typed `sources`/`claims`/`evidence`/`affected_artifacts`/`discovered_by` + `extra` | typed substructures + `#[serde(flatten)] extra` |
| entry `ref` | entry `id` | renamed throughout (metadata, prose, lint, tests) |
| (none) | ledger `delta_ref` | optional `slice_ref` |
| closed enums **fail loud** on typo | **all vocab permissive** — unknown warns (DEC-057-08) | all enums soft (`+ Other(String)`) |

**Round-1 M5 partial reversal (with reason).** M5 was accepted with the nuance
"detector-emitted kinds open (`Other`), but hand-authored `disposition`/status
closed and fail-loud." The canonical schema is new evidence that contradicts the
closed half: *every* drift vocabulary is permissive (warn, not reject). So the
soft-enum treatment now applies uniformly — the open/closed split is dropped.
The graceful-degradation core of M5 stands and is in fact vindicated by the
canonical `extra` + progressive-strictness design. drift-spec mutating verbs keep
the edit-preserving (`toml_edit`) requirement; `extra` is now what they must
preserve (not the retired `observed`).

Heresiarch's deliberate divergences are **kept** (not schema errors): the
directory-entity layout (`.doctrine/drift/<n>/` + sister toml/md vs the
canonical single `DL-NNN-<slug>.md`), the slug+title naming, and lifting
`analysis` out to prose (the canonical also keeps long narrative as freeform
markdown after the fence, so the split is consistent).

No code touched — drift remains deferred (registry-gated). This is a
note-faithfulness fix against the now-authoritative schema, committed separately.
