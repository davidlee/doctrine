# Review RV-241 — design of SL-195

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Design-aspect inquisition on the LOCKED design + 3-phase plan of SL-195 (installer
dual-mode: `--dev` marketplace source + `.mcp.json` POL-002 portability fix),
before any code is written. Posture: `--raiser inquisitor`. External adversary:
codex (GPT-5.5), read-only, given design/plan/notes + the boot.rs / install.rs
seams as ground truth.

Lines of attack:
1. SPEC-009 idempotency of `plan_mcp` once the command goes env-form.
2. R1 ownership-predicate widening (`is_doctrine_mcp_entry`) + the outcome String.
3. STD-001 magic-string discipline for `${DOCTRINE_BIN:-doctrine}` / `doctrine@doctrine`.
4. F3 flag surface — bare `--dev` vs explicit `--marketplace-source`.
5. `--dev` abs-root leak onto any committed surface (POL-002).
6. PHASE-03's deferred refresh verb — sound deferral or hidden blocker.
7. Silent-failure paths in the install shell-outs; unowned invariants; test gaps.

Invariants pinned: POL-002 (no host abspath in a tracked file), STD-001
(single-source named consts), SPEC-009 (idempotent install), the design's own
`baked ⟺ gitignored` invariant, and INV-1/2/3 from design §5.5.

## Synthesis

**Verdict: HERESY CONFIRMED. The "locked" design is not clean — one blocker and
four majors survived cross-examination.** The two deliverables are directionally
sound (the POL-002 env-form fix and the `--dev` source axis are the right shape),
but the design and plan named only the *obvious* arms of the change and left the
load-bearing seams unconfessed.

**The blocker (F-1).** The design's own §8 R1 saw the ownership predicate
(`is_doctrine_mcp_entry`) but walked past its twin: `plan_mcp`'s no-op comparator
still weighs the existing entry against `exec.display()` (the abspath, boot.rs:1519).
Switch the desired command to the constant env literal and that equality can never
hold — every reinstall thrashes the committed `.mcp.json` as a bogus refresh, and
the standing `plan_mcp_idempotent_when_current` guard breaks. SPEC-009 idempotency
falls between the two arms PHASE-01 *did* name (EX-2 predicate, EX-3 stale-refresh).
This must be reconciled into PHASE-01 before it executes.

**The majors.** (F-2) The "absolute project root" the whole `--dev` axis and the
PHASE-03 comparator rest on is *assumed, not enforced* — `root::find` returns an
explicit `--path` verbatim (root.rs:23), so a relative path poisons the source
comparison and reopens the idempotency wound in PHASE-03. (F-3) The design misreads
its own manifest: "read `plugins[].name` … both resolve to doctrine" — but
`marketplace.json` holds THREE plugins; the selection rule is unstated and VT-3
hardcodes the answer. (F-4) The presence checks the slice edits are bare substring
greps — `contains("doctrine")` false-matches `doctrine-memory`/`doctrine-partner`.
(F-5) PHASE-03 defers the refresh verb to a live probe while the shell-out policy
swallows failures into `skipped_*` — if the destructive `remove`+`add` branch is
required, a mid-refresh failure leaves doctrine uninstalled yet reports success.

**The minors** (F-6 STD-001 const, F-7 outcome carries/prints the abspath, F-8
`--dev` prompt/reminder text drift) mostly fall out of fixing F-1 cleanly.

**Acquittals (tried, found sound — no charge raised).**
- **F3 flag surface (design §7 D1):** bare `--dev` boolean stands. The external
  adversary concurred with D1 — no concrete extensibility trap; an explicit source
  enum can be grafted later without breaking the boolean. YAGNI holds. Probe closed.
- **Committed-surface abspath leak under `--dev`:** no tracked-file leak found. The
  abs root correctly lands only in per-machine `known_marketplaces.json`; `--scope
  project` writes only the portable `enabledPlugins` key. The invariant
  `baked ⟺ gitignored` and the POL-002 boundary (fix the committed `.mcp.json`
  only; leave pi `mcp.ts` + hooks baked) survived — the earlier draft's re-question
  is put to rest. The *real* `--dev` risk is F-2's comparator, not a leak.

**Sentence (ordered penance).**
1. F-1 (blocker) — fold the comparator fix + an "already-env-form ⇒ None" EX/VT
   into PHASE-01; consume a named const (F-6). Precondition for executing PHASE-01.
2. F-2 — canonicalize the root once before `--dev` source selection; unit with a
   relative `--path`. PHASE-02.
3. F-3 — pin the plugin-selection rule (name == marketplace name), correct §5.1,
   test against a reordered-manifest fixture. PHASE-02.
4. F-4 — exact parsed presence match; fixtures with the sibling plugins. Scope of
   this vs a follow-up IMP is the one open decision for the User.
5. F-5 — commit PHASE-03 to both refresh branches + abort-on-failure for the
   destructive path; state in exit criteria before code locks.
6. F-7, F-8 — carry the written command form in the outcome; render selected
   source + qualified key in prompts/reminders. Cheap, fold into their phases.

Standing risk consciously carried into execution (not blockers): OQ-4 (env-form
`.mcp.json` connects under `/mcp`) and the R4 refresh-verb identity remain live
probes — documented expected answers, observed result recorded in the phase sheet
before dependent code locks.
