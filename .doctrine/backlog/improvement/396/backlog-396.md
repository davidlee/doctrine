# IMP-396: check verdict line must survive a tail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The failure

`doctrine check gate` produces far more output than an agent wants in context,
so the reflex is to bound it:

```bash
doctrine check gate 2>&1 | tail -40
```

The pipe discards the recipe's exit code. `$?` is `tail`'s, which is `0`. A
**red gate reports success** — and in an agent harness the tool result is
annotated "exit code 0", so the lie is confirmed by the harness rather than
caught by it.

Observed at SL-241 `/close` on 2026-08-03. The gate was red on
`every_authored_design_in_this_repo_imports_losslessly` (exit 101); the
invocation reported exit 0. It was caught only because the failure block
happened to fall inside the last 40 lines and got read.

## Why it is worth fixing rather than remembering

Three things compound, and no one of them alone would matter:

1. **The close ritual mandates this exact command.** `CLAUDE.md` names
   `doctrine check gate` as the close-time build-before-validate step, so it is
   run at precisely the moment a false green is most expensive — the transition
   to `done`.
2. **Bounding the output is the correct instinct**, not a habit to train out.
   Gate transcripts are large enough that ISS-219 exists about one of them
   (`worker_commit`'s red-gate refusal embedding ~295k chars). "Don't pipe it"
   fights a real pressure and will lose.
3. **The tail depth is a guess.** Even reading the output does not save you:
   `tail -40` caught the failure block here, `tail -10` would have shown only
   cargo's trailing `error:` lines, and a tail landing after a long green suite
   shows nothing but passes. Correctness currently depends on choosing a number
   large enough for a failure whose size is unknown in advance.

`PIPESTATUS` is the shell answer and it works, but every caller must remember it
every time, forever, and the failure mode when they forget is silent and
confidently wrong. That is the wrong shape for a safety property.

## Direction (not a design)

Make the verdict **positionally cheap** — emit a single terminal line, last on
both paths, so any tail depth ≥ 1 carries it:

```text
GATE: RED — 1 failed (e2e_design_legacy_corpus)
GATE: GREEN — 68 passed
```

A piped read then self-verifies: the presence of `GATE: GREEN` is the signal
rather than the exit code, and its absence is conspicuous. Open questions for
whoever picks this up:

- does this belong to `check` generally (`quick` / `commit` / `gate` share the
  hazard) or to `gate` alone?
- machine-readable token or prose? An agent reads prose fine; a hook would
  prefer a token.
- does the recipe layer (`justfile`) or the `check` proxy verb own the emit?
  The failing leg here was `test-all` exiting 101, so the verdict has to survive
  an inner recipe's non-zero exit — it cannot be the last command of a `&&`
  chain.

## Provenance

- RFC-011 friction record `019fc6de-f4d4-7e80-ba6e-2bd27072ef9e`
  (`.doctrine/observations/records/9e/`), captured at the moment it bit.
- Adjacent, not duplicate: **ISS-219** (`worker_commit` red-gate refusal embeds
  the full check transcript) — same underlying pressure, gate output too large
  to handle whole, arriving at a different symptom.
