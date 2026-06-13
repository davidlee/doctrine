# Ninth Inquisition - SL-056 design (`design.md`, post-round-8 clean target)

Target examined: `.doctrine/slice/056/design.md` as of 2026-06-13, with
`SL-056` read through `doctrine slice show 56`. Supporting doctrine consulted:
ADR-006, ADR-008, SPEC-012, POL-001, the `SL-056` scope, existing
`inquisition.md` / `inquisition-2.md` ... `inquisition-8.md`, scoped memories,
`doctrine worktree --help`, and the SL-055 research base cited by the design.

Style note: the inquisition structure is retained, but theatrical punishment
language is intentionally omitted because POL-001 is required project doctrine.

## Charges

1. **Hook-only mutators are omitted from the worker-mode privilege class.**

   Doctrine violated: ADR-006 D2 says dispatch workers mutate source only and
   every doctrine-mediated write funnels through the orchestrator. The design's
   own guard says worker mode refuses "a write-classed OR Orchestrator Command"
   (`design.md:60`).

   Evidence: `create-fork` runs `git worktree add`, provisions, and writes the
   marker (`design.md:160-176`); the fallback `marker --stamp-subagent` also
   mints marker state (`design.md:303-306`, `design.md:530-531`). Yet the
   `Orchestrator` class lists only `fork`, `import`, `land`, and `gc`
   (`design.md:296-301`, repeated at `design.md:595`), and the verification
   only proves those four are refused (`design.md:621-625`).

   Risk: a worker-mode process can call the hook-facing mutators if they are not
   classified. `create-fork` mutates git refs and worktree directories, exactly
   the reason `fork`/`gc` are not `Read`. The omission leaves a privilege bypass
   on the newest, most trusted path.

   Sentencing: add `create-fork` and `marker --stamp-subagent` to an explicit
   hook/orchestrator write class refused under `worker_mode`; the legitimate
   Claude hook remains allowed because it runs at the coordination root with
   worker mode false. Add `run()`-level invariant tests from both marker and env
   worker signals.

2. **The Claude fork path has no explicit base contract before the worker starts.**

   Doctrine violated: ADR-006 D5/D9 require commit-before-spawn and a
   branch-point check around fork creation; the scope says D9 already mandates
   provision plus baseline verification "before handing the worker its task"
   (`slice-056.md:125-126`).

   Evidence: the codex/pi path explicitly calls `fork --base <B>`
   (`design.md:112-113`). The Claude hook instead runs `git worktree add ... <HEAD>`
   (`design.md:168`) and later admits "No base param reaches the hook"
   (`design.md:272-274`). The O3 verification checks payload shape, hook timing,
   and concurrency (`design.md:680-693`) but does not check that the hook-created
   fork is based at the orchestrator's captured `B`.

   Risk: if session HEAD moves between the orchestrator's base capture and
   WorktreeCreate, Claude can fork from `B+1` while the worker prompt and import
   still reason about `B`. The design will refuse late, after wasting a worker
   run, and the "mirrors `fork --worker`" claim becomes false for base integrity.

   Sentencing: specify how the expected base reaches `create-fork`, or specify a
   hook-time `base-moved` refusal before the worker is born. If the harness makes
   that impossible, state the residual honestly and stop claiming parity with
   `fork --base`. Add a golden where HEAD moves before WorktreeCreate and the
   worker is not spawned.

3. **`assert-marker-absent` is a required transition gate with no owner.**

   Doctrine violated: the thesis says mechanism belongs in CLI verbs, not prose.
   ADR-006 D6a also depends on a clean distinction between solo direct-writer mode
   and dispatch worker mode.

   Evidence: the design requires an `assert-marker-absent` check before "every
   transition of a linked worktree into a direct-writer role, solo `/execute`
   included" (`design.md:75-81`). No owning command, helper, caller, or
   verification is listed in the code impact table (`design.md:590-602`) or the
   verification section (`design.md:604-703`).

   Risk: the gate can be forgotten by the exact skill prose this slice is trying
   to shrink. A stale marker then blocks direct-writer work late, or a direct
   writer starts from an ambiguous role without a pinned refusal/remedy.

   Sentencing: make the gate a named helper or verb and list every caller
   (`/execute` isolated entry, solo worktree handoff, any land/gc transition that
   treats a tree as direct-writer). Add tests for clean direct-writer entry,
   stale-marker refusal with named remedy, and successful entry after
   `marker --clear --operator`.

4. **Required worker-mode observability names a `status` surface that is not in the design's implementation surface.**

   Doctrine violated: the design marks observability as required
   (`design.md:87-89`), and `/canon` treats missing verification of named
   behaviour as suspect.

   Evidence: `doctrine worktree --help` currently exposes only `provision`,
   `check-allowlist`, and `branch-point-check`. The design says "`doctrine
   worktree` status prints ..." (`design.md:87-89`) and lists `status` as `Read`
   (`design.md:300`), but `src/main.rs` impact does not add a `status` subcommand
   (`design.md:595`) and §12 has no status golden.

   Risk: stale markers and env leaks become hard to diagnose, and the required
   operator-facing surface can be missed during planning because it is not in the
   affected-surface table.

   Sentencing: either add `worktree status` to the CLI impact and verification,
   or delete the requirement and point to an existing command. Verification must
   cover no signal, marker signal, env signal, and both signals.

5. **`claude install` is a doctrine-mediated write, but worker-mode refusal coverage omits it.**

   Doctrine violated: ADR-006 D2 says workers perform no doctrine-mediated writes.
   SPEC-012 D3 requires exhaustive write classification with no silent future
   permission.

   Evidence: `claude install` writes skills, agents, and hooks into `.claude`
   (`design.md:492-505`) and keeps `skills install` as a hidden alias
   (`design.md:512-517`). The worker-mode verification proves authoring/status
   writes and the four orchestrator verbs are refused (`design.md:614-625`), but
   it never proves `claude install` or the hidden alias are write-classed under
   worker mode. The code-impact line says "write_class unchanged"
   (`design.md:595`) while also renaming an installer surface (`design.md:596`).

   Risk: a worker can invoke the installer and mutate `.claude` hook/agent files.
   The dispatch import belt rejects `.doctrine/` touches (`design.md:370-372`),
   not `.claude/`, so an installer-created hook delta can ride back as source.

   Sentencing: explicitly classify both `claude install` and the hidden
   `skills install` alias as writes refused under `worker_mode`. Add invariant
   tests for both spellings from a marker worker and an env worker, plus the
   existing alias/same-handler golden for non-worker mode.

6. **The design guesses a future ADR id.**

   Doctrine violated: the boot guardrails say use the CLI and do not guess ids;
   reference forms cite durable canonical ids only after they exist.

   Evidence: G3 says "ADR (new, id via `doctrine adr new` - likely ADR-011)"
   (`design.md:550`). The code-impact table also names `ADR-011 (new)`
   (`design.md:601`). `doctrine adr list` currently shows ADR-001 through
   ADR-010 only.

   Risk: if another ADR is allocated first, the design points future work at the
   wrong identity. Even when the guess happens to be right, the prose normalizes
   id prediction in the one repository that explicitly forbids it.

   Sentencing: replace guessed references with "the spawn-interface ADR allocated
   by `doctrine adr new`" until the entity exists. After creation, update the
   design with the actual canonical id. Verification: no `likely ADR` or
   unallocated `ADR-011` references remain.

7. **The design violates POL-001's prose constraints.**

   Doctrine violated: POL-001 is required and says to avoid "load-bearing" and
   tired physical metaphors such as "seam" and "substrate" unless there is no
   reasonable alternative.

   Evidence: the title itself uses "spawn seam" (`design.md:1`); the prose uses
   "substrate" (`design.md:51`), "env seam" (`design.md:51`, `design.md:152`),
   "load-bearing" (`design.md:509`), and more repeated "seam"/"core" phrasing
   (`design.md:543`, `design.md:550`, `design.md:598`, `design.md:703`).

   Risk: this is an active policy violation in a design that will seed ADR,
   spec, skill, and test names. The vocabulary will spread unless corrected
   before lock.

   Sentencing: replace the banned terms with specific alternatives such as
   "spawn interface", "contract", "boundary", "important", or the concrete
   subsystem name. If a term is kept, justify why no reasonable alternative
   exists. Verification: an `rg` sweep for the POL-001 terms returns only quoted
   historical text or justified exceptions.

8. **The provenance appendix cites the wrong first inquisition artifact.**

   Doctrine violated: authored artifacts should not bear false witness about the
   corpus they reference; the storage rule also treats durable review artifacts
   as committed evidence, not vague memory.

   Evidence: the design says the dispositions live in `inquisition-1.md` through
   `inquisition-7.md` (`design.md:3-4`, `design.md:726-727`). The slice directory
   contains `inquisition.md` plus `inquisition-2.md` through `inquisition-8.md`;
   `inquisition-1.md` does not exist.

   Risk: the next agent following the appendix cannot find round one, and the
   "eight passes shaped this design" summary is needlessly confusing because the
   first report has the unsuffixed filename.

   Sentencing: update the appendix to cite `inquisition.md` as round one and
   `inquisition-2.md` ... `inquisition-8.md` thereafter. Verification: every
   named report in the appendix exists on disk.

## Questions

1. Should `create-fork` and `marker --stamp-subagent` be a distinct hook-only
   write class, or folded into the existing `Orchestrator` class?
2. Can the Claude WorktreeCreate hook receive or derive the orchestrator's
   captured base `B`, or must the design admit a late-refusal residual?
3. Is `worktree status` intended as a new CLI verb, or should observability be
   supplied by an existing command?
4. Should `.claude/` installer output be added to any dispatch import belt, or is
   CLI write-class refusal sufficient for SL-056?

## Pronounce Judgement

The round-8 charges are mostly answered, but the design is not locked. The most
serious remaining issues are privilege classification for hook mutators and the
implicit Claude base contract. The rest are cheaper doctrine/process defects,
but they should still be fixed before plan authoring because they affect command
surface, verification, and durable references.

## Sentencing

1. Fix privilege classification first: include hook mutators and installer
   writes in worker-mode refusal coverage, then add `run()`-level tests.
2. Decide and specify the Claude base contract before O3 becomes a phase gate;
   add a moved-HEAD-before-hook golden.
3. Either add and test `worktree status`, or remove it from the design.
4. Give `assert-marker-absent` an owner and tests.
5. Replace guessed ADR references with an unallocated-ADR placeholder until the
   ADR exists.
6. Apply the POL-001 vocabulary sweep.
7. Repair the inquisition provenance links.

> **HERESIS URITOR; DOCTRINA MANET**
