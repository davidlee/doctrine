# IMP-489: Prompt cascade review: model coverage, currency, and role fitness

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The cascade is the mechanism by which doctrine's guidance reaches a particular
harness, model and stage. It has drifted. This is a bounded review pass, not an
open-ended rewrite: establish for each concern below what is wrong, what "right"
means, and what it costs.

## What is there now

`find install/hymns -type f` → 9 snippets: `README.md`,
`harness/{claude,cursor}.md`, `model/adherence/low.md`,
`model/anthropic/claude-sonnet-4.md`, `model/deepseek/_default.md`,
`preamble/core.md`, `role/worker.md`, `stage/design.md`.

`doctrine prompt model-keys` → `adherence/low`, `anthropic/claude-sonnet-4`,
`deepseek/_default`. Three model snippets in total.

## The concerns

1. **Coverage — non-anthropic models are thinly served.** One *named* model band
   (`anthropic/claude-sonnet-4`), one family default (`deepseek/_default`), one
   trait band (`adherence/low`). Nothing for claude-sonnet-4.5, the GPT/o-series,
   Gemini, Qwen, or Llama; and a family default cannot carry per-model tuning.
2. **Currency — nothing checks the snippets' claims against the world.** The
   Anthropic snippet names models and context/output limits, which drift. The
   only validation verb is `prompt check`, which covers the `replaces` graph and
   the stage vocabulary. IMP-242 proposes a reverse dead-hymn lint and ISS-447
   shows a stage band no caller reaches, so dead and stale snippets are a known
   class with no gate.
3. **Role fitness.** ISS-308 — no role for non-dispatch agents. IDE-053 — a new
   stub agent role and a rewrite of the orchestrator and worker role prompts.
   The `role/` corpus holds exactly one snippet (`worker.md`) while `--role` is a
   required flag, so `orchestrator` resolves through preamble and bands alone.
4. **The selection algebra itself.** RFC-013 (composable trait categories,
   disjunction-via-classification) is open and SPEC-023 governs the mechanism.
   ISS-491 is its concrete defect: an unknown trait value is silently ignored.

5. **Key vocabulary — the caller has to guess.** (Raised by the user during
   SL-282 design, 2026-10-17.) An agent follows the boot floor directive by
   inferring what to pass to `prompt resolve --model` from its own model id, with
   no way to know which terms are understood or carry content: `opus`,
   `opus-5`, `opus-5.5`, `claude-opus-5-5`, `anthropic/claude-opus-5-5`, … — and
   that is just the model facet. `prompt model-keys` lists labels, but nothing
   maps a self-identified model id onto them (no aliasing or normalisation), and
   ISS-491 makes a wrong guess indistinguishable from "nothing for you". The
   remedy likely belongs with the selector algebra (RFC-013, SPEC-023): either
   doctrine resolves a raw model id to keys itself, or the directive tells the
   agent exactly what to pass.

## Ask

A bounded review with findings filed as their own items, plus a decision on
whether the remedy is a slice or a Revision to SPEC-023.

## Related

- RFC-013 — prompt cascade selection algebra (open)
- SPEC-023 — prompt cascade (governing technique spec)
- ISS-308, ISS-447, ISS-491 — live defects in the same mechanism
- IMP-242 — reverse dead-hymn coverage lint (open)
- IDE-042, IDE-053 — model-tier prompts; role prompt rewrite (open)
- `mem.concept.doctrine.hymn-cascade` — the shipped orientation for the mechanism
