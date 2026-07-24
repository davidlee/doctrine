# Review RV-299 — design of SL-227

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial pass (codex/GPT) on the SL-227 design — the combined
Contract A+B slice: the `doctrine library list|tree|show` read surface over
SL-223's publication `Resolver`, plus the minimal-projection install flip
(~80 → 3 base files). Design is at `.doctrine/slice/227/design.md`; scope at
`.doctrine/slice/227/slice-227.md`. An internal adversarial pass already ran
(design §10 F1–F7); this pass must not merely re-confirm it — attack what it
missed.

### Governing canon (hold the design to these)

- **ADR-019** — embedding ≠ publication ≠ projection (seven independent asset
  properties). **NF-001** (SPEC-026): publication and projection are independent
  manifests, neither derived from the other.
- **ADR-001** — module layering leaf ← engine ← command, no cycles.
- **SPEC-026** (library/publication): FR-001 sole-authority, FR-003 read/error
  classes, NF-002 structural no-write. **SPEC-009** (install): FR-007 three-file
  base, FR-008 first-use roots, FR-009/FR-010 materialize verbs, NF-004 no
  auxiliary-by-default, NF-005 governance-distinct.
- **STD-001** named constants; **POL-002** platform independence; repo denials
  (`print_stdout`, no `unwrap`/`expect` in tests, BTree not Hash, no `as` casts).
- **Behaviour-preservation gate** (AGENTS.md): shared-machinery changes keep
  existing suites green unchanged.

### Lines of attack

1. **The pairing invariant is only phase-ordered, not mechanically enforced.**
   §1/§10 claim "read path (PHASE-01) before any file stops landing (PHASE-02)".
   Is that a real invariant or a convention a future edit silently breaks? Is
   there any executable gate binding the un-projected set to `library show`, or
   only a hand-maintained test list (§9 "no-silent-unreachable gate")? Probe
   whether the gate actually enumerates *the delta* or a hard-coded four docs.
2. **Reachability completeness.** DEC-010 bounds the published set to templates +
   4 reference docs. But the flip stops projecting ~77 files. Does every
   *stops-being-projected* asset become reachable, or only the reference docs?
   What about `install/agents/**`, `install/hymns/**`, `install/workflows/**`,
   `mod.just`, `LICENSE`, `boot-footer.md`, `model-band.md`? If those were never
   projected, say so with evidence; if they were, the pairing invariant is
   violated for them. This is the crux — verify against `install.rs` build_plan
   legs, not the design's assertion.
3. **`[base]` set correctness.** Three files: `.gitignore`, `doctrine.toml`,
   `project-orientation.md`. Is `doctrine.toml` actually needed at rest (dtoml.rs
   reads defaults if absent)? Does shipping a `doctrine.toml` embed change
   root-detection or any read path? Is `project-orientation.md` content specified
   or hand-waved?
4. **Licence classification (D5) is asserted, not derived.** MIT for
   glossary/using-doctrine, GPL for review-ledger/governance. On what authority?
   Is there a rule, or is this a coin-flip that will bite at
   FR-007-licence-provenance? Does the repo LICENSE (install/LICENSE) contradict
   the per-entry GPL calls?
5. **VT-3 change honesty (F1).** The design admits VT-3 changes. Is that the
   *only* behaviour-preservation casualty, or are there other publication/install
   tests that silently flip? Audit §9's "green unchanged" list against the actual
   test bodies.
6. **Deferred requirements as coverage debt.** FR-009, FR-010, and the FR-003
   unsupported-source-type class are all left `pending` (D3/D4/D6). Is the slice
   still *coherent* with three of its named requirements deferred, or is it
   claiming a contract it does not deliver? Does closing SL-227 with these pending
   leave SPEC-009 in a worse-documented state than before?
7. **NF-002 structural no-write proof.** §5.5 claims read-only "by construction"
   proven by a single byte-unchanged test. Is a byte-unchanged test actually a
   *structural* proof, or merely an empirical one that a future write path would
   pass until it doesn't? Is the claim "no import of any mutator" verifiable?
8. **`ContentKind` / `Licence` / `CustomizationStatus` widening.** Additive enum
   widening claimed safe. Does any exhaustive `match` on these enums exist that
   the new variant breaks (a non-additive change masquerading as additive)?

Read each entity via `doctrine <kind> show`, never a single raw tier. Raise every
suspected deviation as a finding framed expected-vs-observed with file:line
evidence. Severity `blocker` only for what must not ship unreconciled.

## Synthesis

**Verdict: strong pass. Eight real findings (3 blocker, 5 major), all verified
against source and integrated; design revised, not merely re-confirmed.** The
external pass earned its keep — it found that the slice's *central* mechanism,
the reachability accounting, was under-specified in a way the internal pass
(design §10 F1–F7) missed.

**The load-bearing outcome — the reachability strategy (X-F1 + X-F4).** The flip
stops projecting the entire `install/` embed (verified `install.rs:1394-1409`),
~69 assets, of which the original design published only ~6. The remedy chosen by
the user was **Option A — publish the full projection complement** ([[DEC-010]]
revised, design D7). This converts the pairing invariant from a hand-listed,
phase-ordered convention into a **derived** set-containment gate
(`{embedded_filenames()} − {base backings} ⊆ {published backings}`, design §9)
that a future added asset cannot silently escape. The rejected alternative
(reachable-elsewhere buckets + allowlist) would have minted a second governed
surface and a curated check the gate could not mechanically enforce.

**Second correctness catch (X-F2).** `seed_authoring_memories` (`install.rs:202`)
runs unconditionally, so "exactly three files" was unreachable as designed;
gated by D8.

**A simplification fell out (X-F3 → reverses internal-F1).** The speculative
GPL calls contradicted `install/LICENSE` (all-MIT). Correcting every published
entry to MIT not only fixes the licence lie but keeps publication VT-3 green —
dissolving the VT-3 casualty the internal pass had flagged. Less change, more
honest.

**Honesty corrections (X-F5/F6/F7/F8).** Error classes that a single adapter
cannot distinguish are now explicitly deferred rather than claimed (D3); scope ↔
design ↔ closure tell one story (FR-009/FR-010 + the two multi-source classes
uniformly `pending`); the `*_is_shipped` embed tests were correctly
reclassified as *unchanged* (they test embedding, not projection); and the base
`project-orientation.md` gained an explicit content/owner/mutation contract.

**Standing risks carried into `/plan` and execution.**
- PHASE-01 now populates a ~70-entry publication manifest (mechanical, but
  bulky) — `/plan` may split it into engine additions / command veneer /
  manifest population.
- D8 gates a live behaviour (the eager orientation-memory seed). Confirm at
  execution that removing the eager seed does not regress boot orientation —
  `project-orientation.md` (base file) must carry what the seed used to.
- The derived reachability gate depends on `base backings` being enumerable
  from `install/manifest.toml [base]`; keep the gate's base-set source identical
  to `build_plan` leg 2's, or the two can drift.

**Tolerated / deferred (conscious):** SPEC-009 FR-009 (D6) and FR-010 (D4);
SPEC-026 REQ-375's *unsupported-source-type* and *metadata-without-bytes* classes
(D3) — all `pending`, each with a follow-up pointer in the slice. No blocker
tolerated unresolved.

**Harvest.** Nothing durable beyond the slice — the findings are design-local
and captured in [[DEC-010]] and the design decisions. [[IMP-309]] (doctor
availability check) already carries the one reusable seam.
