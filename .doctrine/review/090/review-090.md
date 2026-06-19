# Review RV-090 — design of SL-108

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition of SL-108 design, convened at commit `e0f7199f` ("adversarial review
integrated"). The accused: the pi dispatch worker spawn template, D1-D5
decisions, ADR-011/012 compliance, and the shrinkage test cap bump.

### Lines of interrogation

1. **Tool profile heresy.** The design claims a "Full built-in set" of
   `read, bash, edit, write, grep, find, ls`. pi's own README and `--help`
   confess the default is exactly four tools: `read, write, edit, bash`. If the
   accused wanted `grep`, `find`, `ls` they must pass `--tools` — the template
   does not. Did the design merely *assume* the tool set?

2. **agent_end timeout.** D1 declares "the orchestrator should impose a
   deadline" — a vague hand-wave with no value, no mechanism, no recovery path.
   A hung worker that retries indefinitely will make the orchestrator wait
   forever. Is "should" the standard of a *design*?

3. **agent_end extraction edge cases.** The JavaScript one-liner extraction
   path has no fallback for a worker that produced no assistant messages or
   whose last assistant message has no text blocks. A worker that only emitted
   tool calls would yield `undefined` — and then what?

4. **ADR-011 D3 altitude.** Does the design truly occupy the codex/pi column
   at every row (spawn, identity, marker, base, isolation, confinement,
   concurrency, fail-closed) without drop or embellishment?

5. **ADR-006 D9 / ADR-012 provision.** The `cp AGENTS.md` delivery — does it
   sidestep the D9 allowlist provision contract? Does it violate the ADR-012
   integration topology?

6. **Shrinkage cap.** SL-085 shrunk dispatch skills to ≤25 body lines. The
   accused now asks for ≤35 — a 40% loosening for one additional spawn template.
   Was the new line count *calculated* or *guessed*?

7. **Print mode bug.** Was the `--no-session --session-dir` conflict truly
   exorcised from the committed design, or does it still lurk?

8. **Trust posture.** D5's probe was in-jail with `defaultProjectTrust: "ask"`.
   The design admits the posture is "project-config-dependent" but offers no
   escape hatch — no `--approve`/`--no-approve` flag, no config guidance. What
   happens when a project with stricter trust consumes this template?

The Inquisition holds the accused to the invariants primed in the domain map.
Evidence is drawn from pi v0.79.6 `--help` and `docs/rpc.md`, the committed
`design.md`, and the governing ADRs.

## Synthesis

Let the record show: **the design is sound in architecture but sloppy in detail.**
The accused has not sinned against ADR-011 or ADR-012 — the altitude table is
faithfully occupied and the provision contract intact. The mortal sins are of a
lesser stripe: assumption where there should have been verification, vagueness
where there should have been precision, and a cap number plucked from the air.

### The Verdict

**Guilty of one blocker (F-1).** The tool profile claim is *factually false*.
pi ships four tools — not seven. The design's rhetoric about `grep` and `find`
being "included" is heresy of the assumption class. A worker spawned from this
template would lack the very tools the design praises. The penance is trivial —
add `--tools read,bash,edit,write,grep,find,ls` to the spawn template and update
the flag rationale table — but the sin itself betrays a failure to consult
canonical sources before writing.

**Guilty of three major omissions (F-2, F-3, F-4).** The timeout is not a
design decision — it is a wish. The extraction path handles the happy case and
no other. The auto_retry interaction is entirely unexamined. A design that
catalogues flag rationale for `--thinking off` but skips `set_auto_retry` is
unevenly rigorous. Each requires a concrete addition to D1, not a hand-wave.

**Guilty of three minor negligences (F-5, F-6, F-7).** The cap bump is a round
number without arithmetic. The trust posture caveat names the problem but offers
no flag. The AGENTS.md `cp` has no error guard. These are individually small but
collectively betray a design that was written in one confident pass rather than
adversarially stress-tested.

**Guilty of one nit (F-8).** `PI_OFFLINE=1` works but `--offline` is cleaner.

### Clean Charges

The Inquisition finds **no heresy** on the following lines of interrogation:

- **ADR-011 D3 altitude:** The design faithfully occupies every axis of the
  codex/pi column. The spawn template uses `doctrine worktree fork --worker`
  (harness-identical), carries `DOCTRINE_WORKER=1` (optimisation leg), binds
  cwd via `env -C`, and inherits the fork_env contract. No drop, no
  embellishment — the design is a textbook D3 consumer. *Absolved.*

- **ADR-006 D9 / ADR-012 provision:** The `cp AGENTS.md` delivery is
  filesystem-native context, not a provision-contract violation. AGENTS.md is a
  committed file, not gitignored — the D9 allowlist governs irreducible
  gitignored prerequisites. ADR-012 owns integration topology (coordination
  worktrees, branch roles) and does not constrain how worker context is
  delivered. The `cp` is a delivery mechanism, and it is lawful. *Absolved.*

- **Print mode bug:** The `--no-session --session-dir` conflict is absent from
  the committed design at `e0f7199f`. The print mode template carries only
  `--thinking off --no-extensions --no-skills --no-themes`. The adversarial
  review fix is verified. *Absolved.*

### Ordered Penance

Before this design may proceed to `/plan`:

1. **F-1 (blocker):** Add `--tools read,bash,edit,write,grep,find,ls` to the
   RPC spawn template. Update the flag rationale table to note that grep/find/ls
   are explicitly enabled (not default). Update the Tool profile section to
   reflect reality: pi's default is four tools; the design explicitly enables three
   more. **Verification:** `pi --help` confirms the flag syntax; the template
   produces a worker with all seven tools available.

2. **F-2 (major):** Specify a concrete timeout (300s), enforcement mechanism
   (`timeout` wrapping the pi subprocess), abort semantics (RPC `abort` → grace
   → SIGTERM), and retry-interaction rule (wall-clock deadline, inclusive of
   retries). Add to D1. **Verification:** the timeout spec is falsifiable — a
   worker that exceeds 300s wall time is killed.

3. **F-3 (major):** Add extraction fallback ladder: (1) last text block, (2)
   first content block of any type, (3) `status: "no_output"`. Add error_count
   from toolResult messages. Add structured outcome status
   (`success`/`partial`/`no_output`). **Verification:** test the extraction
   against a synthetic `agent_end` payload with (a) no assistant messages, (b)
   tool-call-only assistant message.

4. **F-4 (major):** Decide auto_retry posture. Recommend: disable via RPC
   `set_auto_retry {enabled: false}` — the orchestrator owns retries. Add to
   flag rationale table. **Verification:** the spawn template includes the RPC
   command or a flag decision documented in D1.

5. **F-5 (minor):** Project the pi-augmented skill body line count. Current
   body is 24 lines. Pi arm adds spawn template (~6), flag table (~6), print
   mode (~4). Projected: ~40. Set cap to ≤40 or justify ≤35 by trimming the
   flag table. **Verification:** the cap number in `e2e_skills_dispatch_shrinkage.rs`
   is traceable to a line-count projection in the design.

6. **F-6 (minor):** Add `--approve` to the spawn template (the orchestrator
   trusts its own worker) with a comment that projects with stricter trust may
   override. **Verification:** a worker spawned with `--approve` in RPC mode
   shows no trust hang.

7. **F-7 (minor):** Guard the `cp AGENTS.md` with `|| { echo ...; exit 1; }`.
   **Verification:** the spawn template fails early on missing AGENTS.md.

8. **F-8 (nit):** Replace `PI_OFFLINE=1` with `--offline` in both templates.
   **Verification:** `pi --help` confirms `--offline` is the CLI equivalent.

### Standing Risks

- **Token pressure from raw `agent_end` JSONL** remains the confessed deferred
  risk (scope §4). The design is honest about this; the 15-30K token estimate
  for a typical worker run is significant but not blocking for v1.
- **Matcher drift** (ADR-011 D6 M2) is noted but inapplicable — the subprocess
  arm does not use matchers.
- **`<<<` bashism** is documented as an assumption; non-bash orchestrators must
  use `echo ... | pi ...`. No finding raised — the design is explicit.

### Harvest

- **F-1 → `/record-memory`:** pi v0.79.6 default tool set is `read, write, edit,
  bash` — grep/find/ls require `--tools` flag. Future pi-integration designs must
  not assume the tool set.
- **F-2 → `/record-memory`:** Design-stage timeout specs must be concrete —
  value + mechanism + abort semantics. "Should" is not a design decision.
- No backlog items — all penance is design-stage amendment, not deferred work.

The Inquisition has spoken. Let the design be amended, and let the heretic who
assumed pi's tool set without consulting `--help` reflect on the mortal sin of
assumption.

> **HERESIS URITOR; DOCTRINA MANET**
