# Review RV-075 — design of SL-095

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of interrogation — the doctrine this Inquisition holds the accused to:

1. **ADR-004 §5 (reverse edge)** — `superseded_by` stays typed, verb-written only. D2/D3/D5 correctly preserve this, but does the verb dispatch for POL/STD actually write the reverse edge correctly?
2. **ADR-010 D2 (vocabulary + rules)** — the new `Related` row for Slice/Backlog uses `AnyNumbered`. Does this create ambiguous dual-label Slice→Policy pairs alongside `GovernedBy`?
3. **ADR-010 D4 (verb-written only)** — `supersedes` migrating to `[[relation]]` with `LifecycleOnly` is correct, but does the `read_block` + `IllegalRow` path for `supersession_pair` actually preserve the D4 guarantee?
4. **Corpus integrity** — D2 asserts "all governance supersedes arrays are empty." Is this verified, or could a manual migration delete data?
5. **Schema compatibility** — D6 adds `Superseded` to `PolicyStatus`/`StandardStatus`. Is serde deserialization case-safe? Does `is_hidden` correctly exclude superseded from work-intake?

The areas under scrutiny:
- `src/governance.rs` — `Relationships` struct drop + `relation_edges`/`supersession_pair`/`format_show` rewrites
- `src/relation.rs` — new RELATION_RULES row (Slice/Backlog `Related` `AnyNumbered`)
- `src/policy.rs` / `src/standard.rs` — `Superseded` status variant + hidden set
- `src/adr.rs` → `src/supersede.rs` — `supersede_policy` extraction + `StorageTarget` + POL/STD arms
- `src/main.rs` — `run_supersede` dispatch on `StorageTarget`
- `install/templates/{adr,policy,standard}.toml` — template drop of `supersedes = []`
- Corpus `.doctrine/adr/*/adr-NNN.toml` × 13 — one-time migration surface

## Synthesis

**Verdict: the design is mostly sound but under-specified at its most critical seam — the verb dispatch. The plan exists and addresses most of these gaps.**

### Findings summary

- **F-1 (major, `GovernedBy` + `Related` overlap):** `AnyNumbered` includes ADR/POL/STD, so Slice→Policy pairs have TWO labels (`governed_by` for governance claims, `related` for weak associations). **Acknowledged — doc fix.** The penance: one sentence in D1 acknowledging the overlap and distinguishing semantics.
- **F-2 (major, SL-097 dependency gap):** `supersede_policy()` lives in `adr.rs` today. The design adds POL/STD arms but the code impact summary says `adr.rs` removes it (delegated to SL-097). If SL-097 hasn't landed, POL/STD arms have no home. **Must-fix — make the dependency ordering explicit.** Either add `.after SL-097` or document the interim code location.
- **F-3 (minor, AnyNumbered includes RV):** Defensible, just undocumented. **Acknowledged.**
- **F-4 (minor, IllegalRow visibility):** Design choice is correct, just under-explained. **Acknowledged.**
- **F-5 (major, verb dispatch incomplete):** `run_supersede` references `policy.supersedes_field` which the new `SupersedePolicy` replaces with `storage: StorageTarget`. The F-1 pre-flight (line 4105) and write path (line 4171) need conditional dispatch — the design describes intent but doesn't show the branching. **Must-fix — show dispatch pseudocode for both paths.** This is the highest-impact finding.

### Corpus assertion verified
All 26 governance `supersedes` arrays are empty — the manual migration deletes no data.

### Gate
The two design must-fix findings (F-2, F-5) are load-bearing. Without them, the implementer cannot write the verb dispatch in PHASE-03. The design is approved — F-1, F-3, F-4 are documentation-only; F-2 and F-5 are partially resolved by the plan but need explicit dispatch pseudocode before implementation.
