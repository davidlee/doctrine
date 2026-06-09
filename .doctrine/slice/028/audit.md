# SL-028 audit — Enact ADR-003 reconcile seam and lifecycle states

Hand-authored close-out (no `slice audit` scaffold yet — known CLI gap).
Independent verification of the code-only half (commits `1219a7a..HEAD`,
`e18a5c0`/`3675931`/`9d0c078`/`bf54a52`/`c1381ab`) against `plan.toml` (THE
CONTRACT), `design.md`, ADR-009, and the lifecycle sections of `slices-spec.md` /
`spec-entity-spec.md`. The canon half (ADR-003 amend + ADR-009 + slices-spec
§ Lifecycle authoring) was completed during /design (`3b50786..ee00d94`) and is
out of this audit's commit range; it is cross-checked here only as the authority
the code must match. The auditor did not write this code.

- **Status:** all 3 phases `completed` (rollup `3/3`). `just check` = **fmt +
  plain clippy ZERO warnings + 683 unit (bin) + all e2e green**. Built binary at
  `~/.cargo/doctrine-target-jail/debug/doctrine` (the PATH `doctrine` is the
  stale pre-SL-028 build — jail target-redirect, `mem.pattern.build.jail-target-redirect`).
- **Verdict: PASS-WITH-FINDINGS.** Every EX/VT/VA criterion across PHASE-01/02/03
  is MET, verified by named test, code line, or live binary run. Non-goals
  respected — no scope creep. Conventions honoured (pure/impure split, BTreeMap,
  expect-not-allow, no indexing/as-casts, outbound-only relations, conduct = clean
  engine tier). Two minor doc-drift defects (stale inline vocabulary comments in
  `slices-spec.md` and `spec-entity-spec.md`) and one expected runtime-tier lag
  (the gitignored `boot.md` snapshot) — none block closure; the doc-drift is a
  cheap orchestrator fix. The slice-status `⚠` divergence (`proposed` vs `3/3`) is
  the reconciliation `/close` must perform.

## Criteria coverage

### PHASE-01 — Slice lifecycle FSM + transition verb

| Criterion | Verdict | Evidence |
|-----------|---------|----------|
| EX-1 additive vocab | MET | `SLICE_STATUSES` (slice.rs:509) = the 8+1 set; original six all survive → no migration. |
| EX-2 pure total `classify` | MET | `classify(&str,&str)->Transition` over all 7 variants (slice.rs:596); no clock/disk; date shell-injected in `run_status` (`clock::today()`, slice.rs:359). |
| EX-3 distinct third predicate | MET | `is_transition_terminal`={done,abandoned} (slice.rs:561); `is_terminal_status`={done} (:550) and `is_hidden`={done,abandoned} (:528) left unchanged; pinned by `is_transition_terminal_is_a_distinct_third_predicate`. |
| EX-4 edit-preserving guarded writer | MET | `set_slice_status` (slice.rs:449): classify-gated FromTerminal+SeamBreach refuse, no-op guard, malformed `status`/`updated` refuse, `toml_edit` in-place. LIVE: comment `# keep me` + `[relationships]` survived a `proposed→started` write, `updated` stamped. |
| EX-5 CLI verb wired | MET | `SliceCommand::Status{id,state,note,path}` (main.rs:743) → `run_status` (slice.rs:348). LIVE: `slice status 1 reconcile --note …` printed `audit → reconcile [advance] [self/auto] — closing out`. |
| EX-6 spec + boot prose lockstep | MET (see D-1/D-2) | slices-spec § Lifecycle (line 219+) = new vocab; canary green; boot Core-process PROSE names `…/audit → reconcile → /close` at the SOURCE `install/routing-process.md:29`; routing-TABLE row untouched (`:17` still `/audit → /close`, F2). |
| EX-7 behaviour preservation | MET | rollup/divergence/is_drifted/is_hidden suites green unchanged; full bin suite 683 pass. |
| VT-1 classify table | MET | `classify_forward_chain_is_advance`, `_legit_closure_seam_path_is_advance`, `_named_back_edges`, `_abandon_from_each_non_terminal`, `_noop_when_unchanged`, `_from_terminal_refused`, `_seam_breach_to_reconcile_from_non_audit`, `_seam_breach_to_done_from_non_reconcile`, `_seam_binds_even_from_a_drifted_source`, `_move_out_of_drift_is_skip_not_refused`, `_non_chain_move_is_skip` — all green. |
| VT-2 set_slice_status round-trip | MET | `set_slice_status_advances_and_preserves_comments_and_relationships`, `_noop_holds_content_and_mtime`, `_refuses_from_terminal`, `_refuses_seam_breach`, `_seam_breach_from_a_drifted_source`, `_refuses_malformed_toml` — all green; corroborated by the live run. |
| VT-3 spec-lockstep canary | MET | `slice_statuses_matches_the_spec_vocabulary` (slice.rs:1742) + `slice_status_enum_matches_the_vocabulary` green. NOTE: the canary asserts against a hardcoded literal, not a parse of slices-spec.md — same shape as the pre-existing canary; human-verified the spec § Lifecycle matches. |
| VT-4 suites + lint | MET | `just check` zero warnings; existing suites green. |
| VA-1 boot prose / routing-table | MET | Source prose names reconcile; routing-table skill row untouched (shipped-not-reachable guard, F2). |

### PHASE-02 — Conduct axis (advisory)

| Criterion | Verdict | Evidence |
|-----------|---------|----------|
| EX-1 enums + resolve | MET | `Actor{Agent,Author(serde "self"),Peer,Team}`, `Autonomy{Auto,Draft,Gate}`, `Conduct`, `ConductConfig`, pure `resolve(cfg,state)` (conduct.rs:33-167); `states: BTreeMap` (:113) — HashMap banned. |
| EX-2 toml parse + tolerant defaults | MET | `parse` via `root::find`/`load_conduct` (slice.rs:380); absent file/key → defaults; baked plan&reconcile=gate (conduct.rs:133); unknown-state tolerated (`unknown_state_subtable_is_tolerated_not_errored`). |
| EX-3 status=resolve(from), show=resolve(current) | MET | `run_status` resolves `from` (slice.rs:363); `run_show` Table resolves `doc.status` (slice.rs:916), JSON byte-stable. LIVE: `reconcile → done [advance] [self/gate]` (source `reconcile` gates, F19); `slice show 028` → `conduct: self/auto`. |
| EX-4 commented seed, no entity wiring | MET | `install/doctrine.toml.example` fully commented, git-tracked; NO gitignore-negation, NO manifest/embed wiring (F6); reachable via `install::asset_text` (proven by `shipped_template_is_valid_and_its_defaults_round_trip`). |
| VT-1 parse round-trip + defaults | MET | `full_conduct_table_round_trips`, `absent_conduct_key_parses_to_defaults`, `empty_text_parses_to_defaults`, `plan_and_reconcile_gate_by_default`, `unknown_state_subtable_is_tolerated_not_errored`, `canary_documented_shape_parses`. |
| VT-2 precedence + resolve(from) | MET | `override_beats_default_per_field`, `override_beats_baked_gate_default`, `status_line_carries_the_source_exit_posture`. |
| VT-3 status/show posture string | MET | `status_line_carries_the_source_exit_posture` asserts `reconcile → done [advance] [self/gate]`; `format_show_renders_identity_and_scope_body` asserts `conduct: self/auto` (show side). |
| VT-4 lint + suites | MET | `just check` zero warnings. |

### PHASE-03 — Requirement/coverage enums (vocabulary stubs)

| Criterion | Verdict | Evidence |
|-----------|---------|----------|
| EX-1 ReqStatus additive + documented | MET | `ReqStatus{Pending,InProgress,Active,Deprecated,Retired,Superseded}` kebab serde (requirement.rs:91); meanings in doc-comment (:73-78). |
| EX-2 CoverageStatus + self-clearing dead_code | MET | `CoverageStatus{Planned,InProgress,Verified,Failed,Blocked}` (requirement.rs:145) with `#[cfg_attr(not(test), expect(dead_code, reason=…))]` — scoped to non-test so the reason is fulfilled exactly where the gate's plain clippy applies (dead-code-self-clearing-leaf). |
| EX-3 NO derivation | MET | grep across src/: no `f(coverage)`/`from_coverage`/`sync`/reconcile mapping between the two enums; the only `sync` hits are the unrelated memory-corpus sync. Doc-comment (requirement.rs:84) states the divergence explicitly. |
| EX-4 spec-entity-spec § Lifecycle | MET (see D-2) | spec-entity-spec.md:230 + § Lifecycle:327-336 carry the full vocab + meanings incl. in-progress/retired. |
| VT-1 ReqStatus new-variant serde+render | MET | `req_status_new_variants_serde_round_trip_and_render`. |
| VT-2 CoverageStatus serde all-five | MET | `coverage_status_serde_round_trips_all_five_variants`. |
| VT-3 lint + req suites | MET | `just check` zero warnings; requirement suites green. |

## Non-goals — respected (no scope creep)

`git diff --stat 1219a7a..HEAD` = 8 files, exactly the planned affected surface
(slice.rs, conduct.rs, requirement.rs, main.rs, slices-spec.md, spec-entity-spec.md,
routing-process.md, doctrine.toml.example). Confirmed NOT built:

- **No `slice reconcile` CLI verb** — `slice --help` lists only `status` (its help
  text mentions reconcile as a *state*, not a verb).
- **No `/reconcile` skill** — none under any `*/skills/` tree.
- **No reconcile artefact / entity kind** — no top-level kind, no new manifest dir.
- **No coverage derivation / registry / coverage blocks** — `coverage` appears only
  as the stubbed `CoverageStatus` enum in requirement.rs; `ReqStatus = f(coverage)`
  is absent by design (EX-3).
- **No conduct enforcement** — `resolve` is read-only; nothing gates a write (the
  verb writes regardless of posture; the seam refusals are *structural*, not conduct).

## Convention findings

None. Verified:
- **Pure/imperative split** — `classify`, `resolve`, `parse`, `status_line` are
  pure; the only impurity (`fs::read_to_string`, `clock::today()`) lives in the
  `run_status`/`load_conduct`/`read_status` shells. conduct.rs imports no command-
  tier module (engine tier, ADR-001 — no upward edge).
- **Lint denies** — `BTreeMap` (not HashMap); no `#[allow]` anywhere in the touched
  files (expect-not-allow honoured, the one `expect` carries a reason); no indexing-
  slicing, no `as`-casts in the new code (the only `HashMap`/`as` tokens are inside
  comments). Plain `cargo clippy` zero warnings.
- **Outbound-only relations** — `Relationships{specs,requirements,supersedes}`
  unchanged; nothing reciprocal added.
- **Storage rule** — `doctrine.toml.example` is structured config; `boot.md` stays
  gitignored/derived (`git check-ignore` confirms, untracked); no queried data in
  prose.

## Canon ↔ code coherence

No drift between canon and code:
- **`classify` FSM ↔ ADR-009 §1 mermaid / design §5.4** — forward chain
  `proposed→design→plan→ready→started→audit`, seam advances `audit→reconcile→done`,
  back-edges `audit→{started,design}` and `reconcile→{audit,design}`, abandon from
  any non-terminal, FromTerminal+SeamBreach refusals. The `reconcile→design`
  model-gap escalation (mermaid forward-styled) is classified `BackEdge` in code —
  consistent with design §5.4 / ADR-009 §1 ("falls back to … design (redesign)";
  it is a corrective walk-back, not a forward advance). Coherent.
- **`SLICE_STATUSES` ↔ slices-spec § Lifecycle** — identical 8+1 vocabulary; the
  canary pins it.
- **Conduct axis ↔ ADR-009 §2** — actor×autonomy, advisory/invoker-blind, baked
  defaults self/auto except plan&reconcile=gate, autonomy=exit-semantics (F19),
  Author⇒"self" rename. Exact.
- **Two-enum model ↔ ADR-009 §3** — ReqStatus 6-set and CoverageStatus 5-set match
  the ADR's stated vocabularies verbatim; the no-derivation stance (explicit-
  reconcile-vs-derive) is honoured (EX-3) and documented in-code.

## Divergences / defects (for the orchestrator's fix step)

- **D-1 — stale inline vocabulary comment in `slices-spec.md` (LOW, doc-drift,
  FIX).** The TOML *example block* at `slices-spec.md:92` still lists the OLD
  six-token set `# proposed | ready | started | audit | done | abandoned`
  (missing design/plan/reconcile), and line 104 still reads "transitions are by
  hand" — both contradict the now-correct § Lifecycle (line 219+) and the new
  `slice status` verb. The authoritative section is right; only the illustrative
  comment lagged. Cheap one-line fixes; not load-bearing on any test.
- **D-2 — stale historical enum line in `spec-entity-spec.md` (LOW, doc-drift,
  OPTIONAL).** Line 162 (the "requirement parse struct" descriptive prose) still
  shows `ReqStatus { Pending, Active, Deprecated, Superseded }` (pre-SL-028).
  The authoritative § Lifecycle (line 327+) and the field example (line 230) carry
  the full new vocab. Arguably a frozen historical note; recommend updating for
  consistency, low priority.
- **D-3 — in-tree `boot.md` snapshot is stale (EXPECTED runtime-tier lag, NOT a
  defect).** `.doctrine/state/boot.md:33` still reads `/audit → /close` while its
  SOURCE `install/routing-process.md:29` reads `/audit → reconcile → /close`.
  boot.md is gitignored/derived and regenerates from the embed via `doctrine boot`;
  the commit (`3675931`) correctly edited the source. This is the documented ≤2-
  session lag, resolved by a `doctrine boot` regenerate (the /canon freshen ritual),
  not a code fix.

## Reconciliation needed (the /close lifecycle move)

`slice list` shows **`SL-028  proposed ⚠  3/3`** — the rollup reports 3/3 phases
complete, but `slice-028.toml` `status` is still `proposed` (the ⚠ divergence).
`/close` must advance the authored status through the FSM to reflect reality.
Because the closure seam is now structurally enforced, the terminal `done` is
reachable only via `…→ audit → reconcile → done`; the orchestrator drives the
`slice status` verb across that path (the verb this slice shipped — the lifecycle
gap it closes, dogfooded). Independent auditor did NOT mutate status — that is the
/close step's job. Fold D-1/D-2 doc fixes before or during close.
