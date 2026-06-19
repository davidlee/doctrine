# Review RV-091 — reconciliation of SL-108

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-108 is a two-phase slice adding pi dispatch worker integration via RPC mode.
PHASE-01 updated the dispatch-subprocess skill with a pi RPC spawn template.
PHASE-02 ran an e2e cadence validation (fork→marker→spawn→agent_end→import→commit)
with pi v0.79.6 in the project jail.

This audit probes:
1. **Design→implementation fidelity** — does the skill template match the design,
   and does the design match pi v0.79.6 behaviour?
2. **Spawn template correctness** — does the documented spawn command actually
   produce a running pi worker?
3. **RPC protocol conformance** — are the RPC commands and flags correct for
   pi v0.79.6?
4. **Gate health** — `just check` green, test pass, lint zero.

Invariants held:
- The spawn template in SKILL.md MUST produce a running pi worker when copied into
  a shell (VA-2).
- The design.md spawn template MUST match the SKILL.md template.
- RPC commands MUST use the correct format per docs/rpc.md.
- No `extension_ui_request` dialogs (select/confirm/input/editor) MUST block
  headless execution (EX-4). Fire-and-forget widgets are acceptable noise.
- Timeout enforcement via `timeout 300` MUST kill a hung worker (VA-4).
- The extraction fallback ladder MUST produce correct statuses for all three rungs
  (VA-3).

## Synthesis

SL-108 was a low-cost integration: one skill doc amendment (PHASE-01) and one
e2e validation exercise (PHASE-02). The gate is green: 1840 tests, zero lint
warnings, `just check` clean.

PHASE-02 revealed two design defects in the spawn template:

1. **Heredoc vs RPC stdin lifecycle** (F-1, blocker). The skill template and
   design both use a bash heredoc to deliver JSONL to pi's stdin, but pi v0.79.6
   RPC mode exits on stdin EOF even with an in-flight model call. The heredoc
   closes stdin after writing, so pi exits before the model responds. This is a
   fundamental mismatch between the documented spawn interface and pi's actual
   behaviour — the template as written does not produce a working worker (VA-2
   fails).

2. **set_auto_retry command format** (F-2, major). A secondary format error in
   the same template block: the RPC command uses a `request` wrapper that pi
   doesn't recognise. Auto-retry is silently never disabled.

3. **subagent-async widget noise** (F-3, minor). The pi-subagents package emits
   a fire-and-forget widget on the RPC stream. It doesn't block execution
   (EX-4 passes) but adds unexpected events to the output.

The extraction ladder, timeout enforcement, worker cwd binding, and import
mechanic all validated successfully. The delta was clean; the gate stayed green.

Standing risk: the spawn template needs amendment before the pi arm can be used
in production dispatch. Both findings are delegated to `/reconcile` for direct
edit of design.md and SKILL.md. The risk is bounded — no codex arm regression,
no CLI changes, no behavioural change to any existing dispatch path.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-1**: Rewrite the spawn template in `design.md` §Spawn template (pi arm) and
  `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` §Spawn — pi arm to handle
  RPC stdin lifecycle. Options:
  a) Named pipe (fifo) pattern: `mkfifo /tmp/pi-in; { printf '...'; sleep 300; } > /tmp/pi-in &; pi --mode rpc ... < /tmp/pi-in`
  b) Switch to print mode: `pi -p ...` (simpler, fire-and-forget; no structured agent_end)
  The design.md D1 text describing the heredoc as the delivery mechanism also needs update.

- **F-2**: Fix `set_auto_retry` format in both files:
  Change `{"type":"request","method":"set_auto_retry","params":{"enabled":false}}`
  to `{"type":"set_auto_retry","enabled":false}`

- **F-3**: Document in SKILL.md that `extension_ui_request` events from installed
  packages (subagent-async widget) are expected on the RPC stream and should be
  filtered/ignored by extraction pipelines. No code change required.

### Governance/spec (REV)

None — all findings are per-slice scope.

## Reconciliation Outcome

### Direct edits applied
- **F-1 + F-2**: Replaced heredoc spawn template with fifo pattern in both
  `.doctrine/slice/108/design.md` §Spawn template (pi arm) and
  `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` §Spawn — pi arm (RPC mode).
  Corrected `set_auto_retry` format from `{"type":"request",...}` to
  `{"type":"set_auto_retry",...}`. Updated flag rationale table (design.md) with
  `sleep 300` and `extension_ui_request` notes.
- **F-3**: Documented `extension_ui_request` widget noise expectation in
  SKILL.md prose. Tolerated — no code change.
- Shrinkage test cap remains ≤40; trimmed prose to fit. `just check` green (1840
  tests, zero lint).

### REVs completed
None — all findings were per-slice direct edits.

### Withdrawn / tolerated
- F-3: tolerated — fire-and-forget widget noise from pi-subagents package;
  rationale in finding disposition.
