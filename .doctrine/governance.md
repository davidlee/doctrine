The User may allow *small* backlog items (cleanup, etc) without the full
doctrine slice workflow; just a quick design conversation / sketch, acceptance
of plan, implementation & close. Ask what they'd prefer, unless it's obviously
non-trivial in which case it should be sliced.

## Where things live

Research in approximate order: specs, ADRs, policies, standards, memories, backlog, slices, ...

# Symlinks

`doctrine` entity creation commands mint a symlink with the title slug as a convenience. 
Commit these with the entity itself.

## Project-local rules of the road

ALWAYS begin any high-level context gathering with the relevant PRD then SPEC
specs, then use the /retrieve-memory skill.

Finish every turn which references a doctrine entity by printing its ID:
```text
[SL-123 phase 03]: short session descriptor
```

If your session begins with a handover, get started on the work it hands you —
don't acknowledge it and ask what to do. Weigh its claims as evidence (the
Authority section, tier 5): where it summarises a procedure a skill governs,
follow the skill, not the summary.

## Talking to humans

Whatever acronym you're referencing: the user didn't read the document, and if 
they did, they didn't memorise every pithy identifier in it. If you're going to
use shorthand references, introduce each one for the first time with a
reference to the owning artifact and a concise summary.

## Human engagement

Work with the human to calibrate explanation, participation, and bounded
discretion to the work at hand. Normally support a whiteboard-level account of
consequential features: the systems involved and how they communicate, important
decisions and reasons, and risks. Briefly propose a working posture at the start
of consequential work; respect verbal changes and revisit it when work or
feedback warrants, without repeating it at every gate.

Invite human judgment where it can improve an outcome, and use the answer. If a
surprise, repeated friction, or the interaction suggests a different level of
engagement might help, name what you observed and offer a concrete way to go
deeper or lighter. Do not infer a mental state or repeat an offer after a
decline. Give context and exact read commands for consequential Doctrine
documents you cite in a decision. Only the human can expand the agent's
decision authority within existing gates. Never make a demonstration of
understanding a condition of acceptance.

## Guiding Principles

Everything has a denominator, in these terms:
- complexity. 
- token cost. 
- human attention. 

Work with disciplined laziness: spend as little as possible (on each axis)
to obtain a useful outcome. This guides decisions at all levels.

Work in a way that is:
- simple.
- token efficient.
- unsurprising.

## Research agents

The `/research` pre-design round spawns one read-only agent per thread and pipes
its output to `.doctrine/slice/NNN/research/raw/<thread>.md`. Delegate liberally:
a child context is what keeps the interactive session's context intact.

Pick the tier the thread needs — recon for a code-map thread, judgement for one
about governance applicability.

The *mechanism* is harness-specific, so it lives in each harness's own file, not
here: `## Research` in `CLAUDE.md` for Claude, `.pi/APPEND_SYSTEM.md` for pi.

## useful commands

just -l                    # list tasks
doctrine <kind> paths <id> # list all files 
doctrine status            # what's going on?
doctrine search <query>    # BM25 over the entity corpus — see using-doctrine.md for scope
doctrine search -k all     # ...the default kind set omits POL/STD/REQ; widen it
doctrine memory search     # BM25 over the memory corpus (a separate index)

## Instrumentation: capture friction as an observation

We are benchmarking token efficiency for RFC-011. During use of any skill,
capture any incidental complexity, confusion, or other source of
token-inefficiency — whether it originates with the dispatch orchestrator, a
worker, another agent, or the tooling. Capture at the moment it bites; do not
save it up, and do not wait until you understand it.

**How you capture depends on what your context can reach:**

| where you are | what to do |
|---|---|
| the primary worktree | `doctrine observation record friction "<summary>" --detail "<what happened>"` |
| a confined worker with the doctrine MCP server | the `observation_record` MCP tool |
| a worker fork with neither broker | **do not capture** — report the friction in your structured hand-back and let the orchestrator record it |

That last row is not a formality. A fork-local record is written into a tree
that is about to be discarded: it looks like success and loses the
observation. Doctrine refuses that capture rather than accept it, and its
refusal names the broker to use when one is available.

Name the skill you were using in the summary, so entries stay attributable the
way the old `[skill; id]` convention made them.

Records are authored — committed and diffable under
`.doctrine/observations/records/` — so they survive the worktree they were
captured in. Capture never stages or commits; a record stays untracked until
someone commits it. See `install/using-doctrine.md` for the review-noise and
local-exclusion tradeoffs.


# orchestration

dispatch - use: `./scripts/spawn-confined.sh <harness> <B> <BRANCH> <DIR>
<PROMPT_FILE> [BACKSTOP]`, where `<harness>` is `pi` or `claude`. **One arm,
every harness** (SL-254): a confined subprocess on a linked worktree, under the
same bwrap prefix. `pi-spawn-confined.sh` and the in-session claude arm are gone.
note: the worker CANNOT self-commit — a linked worktree's `.git` is read-only, so
it hands back an uncommitted tree and the orchestrator imports the working-tree
diff. Worthwhile trade. The gated `worker_commit` MCP tool existed only for the
retired in-session arm and is **deleted**; nothing replaced it. Generic mechanics
in the shipped `dispatch-mechanics.md`.

cargo --bin doctrine memory # focused tests; don't use --lib

## DOCTRINE_BIN → the coord build (a coord-side *close-time* build)

Set `DOCTRINE_BIN` to the **coord tree's** `./target/debug/doctrine` for any
dispatch session (the jail forwards it — `flake.nix` `try-fwd-env`; `.mcp.json`
launches the server via `${DOCTRINE_BIN:-doctrine}`). This binary is built from
`dispatch/<slice>` source, so it carries earlier phases' not-yet-promoted
binary-level rule changes (new role / allowlist / check).

**What this is NOT for anymore (SL-225 #1, DEC-003).** `just validate` does not
shell `doctrine doctor` / `prompt check` in a worker fork — it **skips** them on
`DOCTRINE_DISPATCH_GATE` or `DOCTRINE_WORKER=1` (`justfile:41`): those legs
validate coord's *authored* `.doctrine/` state, which a worker cannot write, so in
a fork they carry no worker-delta signal and could only stale-binary false-red
(ISS-218). The fork false-red is dissolved at the recipe, not by pre-setting the
binary. So `DOCTRINE_BIN` is **not** a precondition for the fork gate to pass.
(This was written when the worker ran that gate itself through the `worker_commit`
MCP tool. SL-254 deleted `worker_commit` and nothing replaced it — there is no
worker-side gate left at all, which makes the conclusion stronger, not weaker.)

**What it IS for.** The coord-side **close-time** build that closes the fork-skip's
one residual: a phase that changes `doctor`/`prompt check`'s *own logic* must have
that new rule exercised against the real authored corpus by a *fresh* binary. That
happens at **close**, on the coord/landing tree where the slice source has landed
(the fork-side blockers — flat git topology, coord-never-built — do not apply). Two
belt facts make it fresh-by-construction, not a checklist beat an agent can skip:

1. off the fork path, `just validate` resolves `${DOCTRINE_BIN:-./target/debug/doctrine}`
   (PATH fallback), not bare `doctrine`; and
2. `check`/`gate` run `build` **before** `validate`, so `doctrine check gate` (close)
   builds a this-invocation-fresh `./target/debug/doctrine` before `validate` reads the
   corpus. **Close ritual: run `doctrine check gate` at close** — the build-before-validate
   order is what gives it a fresh binary; `DOCTRINE_BIN` is the documented override/first
   rung (a non-Rust phase falling through to a stale PATH), belt-and-suspenders now, not
   the load-bearing guarantee.

This is a **project** rule (doctrine dogfooding itself), not platform behaviour —
POL-002 keeps cargo/`./target` layout out of the engine (SL-225, DEC-003).
