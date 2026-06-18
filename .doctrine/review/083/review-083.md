# Review RV-083 — reconciliation of SL-099

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Self-audit of SL-099 (Memory read-path relations and data-model hardening).
8 phases implemented via dispatch; all committed to dispatch/099, synced to
review/099, candidate at candidate/099/review-001.

### Lines of attack

1. **Behaviour-preservation at shared machinery** — verify zero edits to
   entity.rs, relation.rs, catalog/, lexical.rs. Existing test suites must
   pass unchanged (I1, I3).

2. **ADR-001 layering** — verify src/links.rs is pure leaf-tier with zero
   upward imports (I2).

3. **Design conformance** — compare each of the 7 objectives against the
   actual implementation: relations in show/retrieve, wikilink extraction,
   backlinks, inspect bridge, lifespan filter + ageing, suggested relations,
   --expand N, validate, --allow-dirty.

4. **CLI surface completeness** — all declared verbs (resolve-links, backlinks,
   validate) and flags (--expand, --allow-dirty, --lifespan, --trust, --severity,
   --provenance-source, --review-by) present and wired.

5. **Test coverage** — VT criteria from plan verified via test existence and
   passing results. No test was edited to accommodate new behaviour (EX-8).

6. **Backward compatibility** — existing memories without new fields parse
   identically; unset lifespan = 1.0 = existing sort key unchanged.

### Evidence baseline

- Coordination branch: dispatch/099 at 51a5e382 (9 commits incl. audit fix)
- Review ref: review/099 at 5900423e
- Candidate: candidate/099/review-001 admitted at 49f41684
- Test results: 1724+ unit + e2e tests passing, 0 SL-099 failures
  (1 pre-existing env-only sync test failure in jail)
- Clippy: zero warnings (workspace); cargo fmt: pass
- JS lint: known jail limitation (missing @eslint/js) — pre-existing

## Synthesis

SL-099 delivered all 7 objectives across 8 dispatch phases, landing 2,661
lines of source delta on 11 files with zero regressions against the
shared-machinery suites (entity, relation, catalog, lexical — untouched).

### Closure story

The memory read path now surfaces the authored relation graph and wikilink
cross-references at every read surface: `show` includes relations and
wikilinks sections; `retrieve` blocks carry a `relations:` line; `inspect`
accepts memory refs and renders outbound/inbound/danglers/wikilinks. Three
new CLI verbs (`resolve-links`, `backlinks`, `validate`) and five new record
flags (`--lifespan`, `--review-by`, `--provenance-source`, `--trust`,
`--severity`) fill the gaps identified in the capability-gap analysis.

The lifespan/ageing model is behaviour-preserving by construction: unset
lifespan → factor 1.0 → sort key key-8 unchanged. Existing rank tests pass
with zero edits, proving the design's backward-compatibility claim.

The `--expand N` BFS graph expansion and suggested-relations hint on `record`
complete the interactive exploration surface. All three audit findings (F-1:
stderr→stdout stream confusion in expand separator; F-2: wikilink key-form
targets not resolved to uids in BFS edge set; F-3: leading blank line before
first depth block) were fixed in the candidate at 49f41684 and integrated to
main at d22ab0be.

### Standing risks

- **validate subprocess cost** (R2): each memory's stale check spawns a git
  subprocess via `commits_touching`. Acceptable at ~200 memories; batch as
  follow-up per design W3.
- **dirty-tree attestation trust** (R3): `verify --allow-dirty` stamps a
  less-trustworthy working-tree state. The flag makes the tradeoff explicit;
  the `anchor:` line in `show` surfaces the `checkout_state` kind.
- **suggested-relations BM25 cost** (R1): linear in corpus size at record
  time. Acceptable at current scale; re-evaluate if >100ms.

### Tradeoffs consciously accepted

- Wikilinks are regex-extracted on-the-fly from body text (~0.007s
  corpus-wide at current scale) rather than persisted. This keeps the
  storage model clean (no derived data in authored files) at a negligible
  compute cost.
- `validate` output goes to stdout (one finding per line) not structured
  JSON. The design defers `--json` to a follow-up.
- Suggested relations score `lex_doc` (title+summary+tags) not full body
  text. Deferred per OQ5 — body indexing is a larger change.

## Reconciliation Brief

All three RV-083 findings were `fix-now` (code fixes applied within audit
scope on the candidate branch, admitted at 49f41684). No findings require
design, spec, or governance changes.

### Per-slice (direct edit)

(None — all findings resolved in candidate.)

### Governance/spec (REV)

(None — all findings were code-level `fix-now`, no spec or governance artefact
touched.)

## Reconciliation Outcome

All three RV-083 findings were `fix-now` (code fixes applied within the audit
on candidate/099/review-001 at 49f41684):

- **F-1**: expand_graph blank-line separator changed from stderr to stdout ✅
- **F-2**: BFS edge set resolves wikilink key-form targets to uids ✅
- **F-3**: Leading blank line before first depth block eliminated ✅

No per-slice direct edits needed. No governance/spec REV needed. The candidate
carries all fixes and has been admitted. Reconcile pass complete — handoff to
/close.
