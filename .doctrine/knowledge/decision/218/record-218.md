## Decision

The `REV`'s target set is **six entities**, not four, and it is derived by
**reading the entities**, never by reading `DEC-211`'s count. `DEC-211`'s
*method* stands — one `REV` of this slice's own, plus a by-hand `[[source]]`
anchor sweep. Its *enumeration* is superseded here.

## Why the enumeration is re-derived rather than amended

The survey has under-counted four times, always in the same direction:

| pass | found |
|---|---|
| research | `ADR-011` at four regions |
| drafting | `ADR-011` at five, then eight |
| `RV-355` `F-4` | `SPEC-012` at four falsified requirements, behind a one-line "responsibility 18" |
| `RV-355` `F-5` | `ADR-012` normative on the deleted arm, having been declared "not touched" |
| this re-derivation | `ADR-008` **absent from the set entirely**; `ADR-006` at nine regions, not two; `ADR-011` at eleven, not eight |

A count that has been wrong five times is not evidence; the entities are. The
rule adopted: **re-derive from the corpus at reconcile, and treat any count in
this design — including this one — as a floor.**

*Method, so it is repeatable:* grep the authored governance corpus
(`.doctrine/adr`, `.doctrine/spec`, `.doctrine/policy`, `.doctrine/standard`)
for the deleted mechanisms — `pretooluse`, `subagent`, `nominate`/`denominate`,
`worker_commit`, `arm-spawn`, `verify-worker`, `in-session`, `dispatch-agent`,
`DOCTRINE_WORKER`, marker — then **read every hit in context**, because the
falsifying region is often the one that does not name the mechanism (`ADR-012`
`D3` says "the Claude `Agent` arm", not "subagent").

## The six targets

**1. `ADR-011` — eleven regions** (was eight). Context (21-25); `D1` (39-56);
`D2` (58-74); `D3`'s table (82-92); `D4` (94-111); `D6` (146-207);
Consequences/**Positive** (242-243, the "agnostic floor (marker + provision +
per-wt env emission) is identical … under claude/codex/pi" claim — *new*);
Consequences/Negative (246-255); Consequences/**Neutral** (256-262, "the
subprocess enhancements … are optimisation-tier and **codex/pi-only** until a
free claude env backend (`IDE-004`) lands" — falsified by a subprocess rather
than by `IDE-004`; *new*); Verification (265-279); **References** (283-295 — the
`ADR-008` cross-reference "nested bwrap (D-B3) is the codex/pi OS floor this
ADR's claude cell lacks", plus two stale `mem.pattern.dispatch.*` pointers;
*new*). `D5` (113-144) and `D7` (209-232) are checked and already
falsified/withdrawn on their own terms.

**2. `ADR-006` — nine regions** (was "two corrections at one site"). `D2a`
Signal (59-70 — the `worker_mode` formula itself, "the disk marker is the
harness-agnostic primary, the env leg a codex/pi optimisation (claude has no env
channel)", and the `marker --stamp-subagent` verb-identity exemption); `D2a`'s
unstamped-claude-worker fence bullet (71-83 — the `SubagentStart` stamp-failure
case and the `IMP-052` post-spawn marker check); `D2a` amendment SL-064
(102-115 — permission rests on marker-absence, and the "`env DOCTRINE_WORKER`
must NOT leak" hazard, scoped there to codex/pi, becomes universal); `D2b` body
(116-121); `D2b` note SL-064 (122-138 — the live forward reference to `IMP-065`
as "the real positive-marker close", which `REV-018` closed **obsolete** on
2026-07-02); `D9` amendment SL-056 G2 (230-249 — "the claude rung is
SubagentStart-stamp" and its accepted weaker baseline-verify class); `D9`
amendment SL-064 (251-265 — markerless coordination-tree creation, whose one
difference is that the marker is not stamped); `D10` (270-271 — a candidate
worktree "is unstamped" loses its discriminator); Consequences/Negative
(304-306 — "no harness enforcement (D2b) … rests on the CLI guard (D2a) plus the
prompt contract").

Checked and **not** changed: the G2-revert rationale (84-93) stands as history;
Verification (326-327) asserts refusal under `DOCTRINE_WORKER=1` and becomes
*more* accurate, not less; References (365-367) already records `IMP-065`
retired.

**3. `ADR-008` — seven regions. A sixth target entity, absent from every prior
survey.** Context (48-52 — "the agnostic worker-sole-writer guarantee rides the
**disk marker + the `import` `.doctrine/`-rejection belt**"; the marker leg goes
and the belt becomes the whole of it); `D-B3` (97-110 — "**codex/pi-only**:
claude's `Agent` tool is not a subprocess and cannot be wrapped … so its
worker-sole-writer stays accident-fenced + prompt-enforced", plus "it **ro-binds
the marker only**"); `D-B6` (111-145 — the entire nominated-unjailed-orchestrator
mechanism: the `SubagentStart` nominate hook, the `PreToolUse(Agent)` gate,
`SubagentStop` hygiene, invariants `I1`/`I2`, and the ledger's "confined Mode-B
orchestrator is the reversible escape hatch", which `DEC-217` retires);
Consequences/Negative (176-181 — `D-B6`'s "standing obligation … forever", which
dissolves); Verification (202-206 — the `I1` doctor-check fixture); `N1`
(231-246 — the sanctioned `worker_commit` exception to the `PreToolUse` jail
wall, whose wall *and* tool are both deleted); References (218-221 — "`D2b` is
discharged here only where `D-B3` lands (codex/pi, userns-permitting) … with
`D2a` — the marker-primary CLI guard — as the agnostic fallback").

`ADR-008` is where the deletion is most load-bearing and least visible: it is
project-local, so no `[[source]]` anchor and no `spec validate` leg points at
it.

**4. `ADR-012` — three regions.** Decision 3's harness-synthesis rule (73-78 —
"on arms that do **not** return a per-worker fork branch — the Claude `Agent`
arm (`ADR-011`), where the worker delta lands directly onto `dispatch/<slice>`";
the case goes empty); Verification `M3` (258-259 — the same rule as a fixture);
the *Boundary — what stays on ADR-006* section (163-166), which restates `D2a`'s
"write-permission rests on marker-absence (positive signal)".

The design's claim that `ADR-012` is "not touched, and this is load-bearing: it
is what keeps the `REV` inside the surveyed set" **inverts**. What the retained
import transport preserves is `ADR-012`'s *integration* decisions (D4/D5, the
two-stage projection). The arm-topology clause is falsified **regardless of
transport**, because the falsifying fact is that the collapsed arm now returns an
orchestrator-created fork branch at all. The amendment is in-place, on the
precedent `ADR-011` already sets.

**5. `SPEC-012` — four requirements and six prose regions** (was: responsibility
line 18, the stamp). `REQ-192` **rewrite** (the `worker_mode` formula and the
marker-absent fail-closed leg); `REQ-248` **rewrite** (`fork` "stamps the worker
marker before any spawn window"); `REQ-250` **rewrite** (`land` refuses a
"marker-bearing (`dispatch-fork`)" fork — substituted by the branch-shape
classifier); `REQ-252` **narrow** ("delivery is subprocess-only (codex/pi)" — the
parenthetical goes stale once every arm is a subprocess; the requirement itself
survives). Prose: responsibility line 18 (`fork = create + provision + stamp +
emit per-wt env`); responsibility line 19 (the disk-marker-primary guard); the
Overview keystone; the *worker-mode guard* body section; the *per-harness
altitude* section's whole **claude** bullet; the *Concerns* "claude altitude is
weaker" bullet.

Checked and **unaffected**: `REQ-189`, `REQ-190`, `REQ-191`, `REQ-193`,
`REQ-194`, `REQ-195`, `REQ-196`, `REQ-249` (the import belt), `REQ-251`.

**6. `SPEC-021` — four requirements and two responsibility lines** (was two
requirements). `REQ-288` **retire** (its premise is a choice between two arms;
retirement also moots `ISS-347`, which records that the requirement states the
env-marker as "`.claude/` presence" while the router actually tests
`CLAUDECODE=1`); `REQ-291` **rewrite** (one column, not two — and it carries the
enforcement-altitude change `RV-355` `F-3` found: the worker-side **mutating**
`CheckKind::Commit` gate is replaced by an orchestrator-side **non-mutating**
`CheckKind::Prove` gate); `REQ-384` **narrow** and `REQ-387` **narrow** (the
`spawned` / `worker-committed` positions lose their production writers, and the
"through mediation" leg loses its entry point — `DEC-217`); responsibility line
16 (arm routing) and the first half of line 19 (enforcement altitude).

Checked and **not** changed: responsibility line 15's funnel cadence — the belt
does not re-home, because the incumbent import transport is retained
(`DEC-213`); `REQ-335` stays `pending` **as a contract** while its one partial
implementation retires.

## The anchor sweep (unchanged from `DEC-211`)

`spec-021.toml:30`'s dangling `[[source]]` at
`plugins/doctrine/skills/dispatch-agent/SKILL.md` resolves by deletion;
`spec-012.toml:30` and `spec-022.toml:45` are comment lists naming
`pretooluse.rs` and `subagent.rs`. `spec validate` does not catch a dangling
anchor, so this is by hand and is a named `REV` item.

## Negative control

Checked and **not applicable**: `ADR-020`, `SPEC-030` (whose §195-196 states
positively that "harness-specific in-session subagent identity is not part of
the capsule contract"), `REV-046`, `POL-002` (satisfied, not strained),
`STD-001` (governs deletion thoroughness, not a `REV` target), `SPEC-011`,
`SPEC-023`, `SPEC-024`, `ADR-018` — the last four are false positives on
"in-session" / "nominated" / "denominated" in unrelated senses, and are the
sweep's positive control that the grep discriminates.

`funnel-machine.md` is a **generated** artefact pinned byte-for-byte to
`src/funnel_machine.rs`'s table. The table does not change (`D6` retains the
machine), so it is not a `REV` target — it moves with code or not at all.

## Consequences

The governance half grows again, exactly as the scope's "this is the larger
half" warned, and the growth is concentrated in the two entities nobody had
opened: `ADR-008` and `ADR-006`'s decision body. The plan's governance phase
must be sized against six entities and roughly forty regions, not four and
twelve.

Supersedes `DEC-211`'s enumeration. Raised by `RV-355` `F-4` / `F-5`.