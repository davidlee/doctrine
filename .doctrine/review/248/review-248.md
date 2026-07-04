# Review RV-248 — reconciliation of SL-201

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Mode:** conformance (post-implementation self-audit). **Surface reviewed:**
non-dispatched — direct edits on `edge` (commits `cde24a6e` scope → `5b337cb9`
design → `37f78b67` plan → `f5085dcb` feat). No candidate branch.

**Subject.** SL-201: `--focus` accepts memory refs; `doctrine onboard` verb;
nodes render titles not uuids.

**Lines of attack:**

1. **Conformance algebra.** `slice conformance 201`: 3/3 design-target code
   files conformant (`cli.rs`, `guard.rs`, `map.rs`), 0 undelivered. Undeclared
   cell = (a) SL-201's own authored doctrine edits (design.md, slice-201.toml,
   gotcha memory) — expected, design-target is code-only; (b) foreign churn
   (SL-198/199, ADR-008, RV-247) — shared-tree bleed in the delta oid range.
2. **Design↔impl fidelity.** Does the shipped code match the *corrected*
   design (D1 CLI verb, D2 server-side focus, `collect_all` reuse seam)? The
   design was corrected mid-flight (F1 below) — is the correction fully
   propagated across scope/design/impl?
3. **Verification honesty.** VT-1/2/3 PASS + attributed. The VH (browser
   title-render) could not run headless — is the residue framed, not hidden?
4. **Behaviour preservation.** Existing focus tests (`valid_focus_sl001`,
   `invalid_focus_*`, `map_serve_path_flag_*`) stay green unchanged (EX-3).

**Invariants held:** pure value_parser (no disk in `validate_focus`);
exhaustive guard match (compiler-forced `Onboard` arm); STD-001 single-source
`ONBOARDING_MEMORY_KEY`; frontend untouched (D2).

## Synthesis

SL-201 lands clean. The delivered code footprint is exactly the three
design-target files (`map.rs`, `cli.rs`, `guard.rs`) — conformance reports 3/3
conformant, 0 undelivered, and 0 undeclared *code* paths. VT-1/2/3 pass and are
git-attributed to the slice diff; `just gate` is green; the existing focus
tests (`valid_focus_sl001`, `invalid_focus_*`, `map_serve_path_flag_*`) stay
green unchanged (EX-3 behaviour preservation holds).

The one substantive story is the **reuse-seam correction**, already caught and
fixed during execution (phase finding F1): the design named
`resolve_inspect_uid`, which is items-only and silently fails the *shipped*
onboarding key `mem.signpost.doctrine.overview`. The false-positive trap was
verifying resolution via `memory show` — whose `run_show` carries a shipped
`.or_else` fallback that `resolve_inspect_uid` lacks. The fix (`collect_all` +
`resolve_memory_from_all`, the items+shipped union `run_resolve_links` uses) is
shipped, the design is corrected inline, and the gotcha is recorded as durable
memory `mem.fact.memory.key-resolution-items-vs-shipped`. Audit residue: the
**scope** `## Context` prose (F-1) still names the abandoned resolver — a
prose-only drift routed to reconcile.

**Standing risks / accepted tradeoffs:**

- **VH residue (F-2, tolerated).** The browser title-on-node half was not
  executed — no graphical host in the jail. Framed, not hidden: the render path
  is pre-existing frontend behaviour (D2, `web/map/src` untouched); only the new
  resolution + error-before-bind surface is SL-201's, and that ran headless.
  Recommend the user run `doctrine onboard` on a graphical host to confirm the
  overview node shows its title (not the uid).
- **`--focus` inert without `--open` (phase F2).** Pre-existing semantics —
  `map_url` inside `if config.open` is the sole focus consumer; no server-side
  initial-focus injection. `onboard` forces `open=true`, so its focus always
  lands. Not introduced here; noted so a future `map serve --focus … ` without
  `--open` isn't mistaken for a regression.
- **Conformance noise (F-3, aligned).** Undeclared cell carried other agents'
  shared-`edge`-tree churn (SL-198/199, ADR-008, RV-247) plus SL-201's own
  authored doctrine edits. Expected: design-target is code-only; a shared tree
  bleeds foreign paths into the record-delta oid range. No code creep.

Close story: no blocker, no code fix owed. One per-slice prose edit (F-1) is the
entire reconcile surface — no REV, no governance touch.

## Reconciliation Brief

### Per-slice (direct edit)
- **F-1 — `slice-201.md` ## Context:** rewrite the "Key→uid resolution already
  exists (`memory::resolve_inspect_uid` + `MemoryRef::parse` …)" sentence to name
  the delivered seam — `collect_all` + `resolve_memory_from_all` (items+shipped
  union) — and drop the `resolve_inspect_uid` claim. Keep the
  `build_memory_key_map`-does-not-exist note (still true). Aligns scope narrative
  with the corrected `design.md § Reuse seam` and the shipped code.

### Governance/spec (REV)
- None. No ADR, policy, standard, or spec finding.
