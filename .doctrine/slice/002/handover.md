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
