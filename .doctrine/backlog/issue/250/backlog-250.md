# ISS-250: dispatch_reap not-landed refusal prescribes dispatch_reap, routing the operator to the CLI gc escape

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`dispatch_reap` refusing a fork that advanced past its imported tip returns, in
`detail`, verbatim:

> fork dispatch/sl230-p01 has not provably landed: **if this fork is
> funnel-managed, reap it with `dispatch_reap`** (the funnel record is its
> landing proof); otherwise `--force` to reap knowingly, or
> `--superseded-head <SHA>` to assert it is spent-and-abandoned. A squash-merge
> cannot be certified — re-land via `worktree land` (--no-ff).

The caller **is** `dispatch_reap`. The first clause is circular, so the only
actionable advice left in the sentence is the `--force` / `--superseded-head`
pair — both of which exist on CLI `worktree gc`, the one landing consumer that
still runs the patch-id oracle (NEW-OQ-C).

## Observed consequence (not hypothetical)

SL-228 PHASE-07's memory-blind benchmark, S5 scenario. The harness advanced
`dispatch/sl230-p01` by one commit after import, so the D9 conjunction should
refuse — and it did, correctly (RV-308 F-1 holds; the branch survived the
refusal). The blind operator then followed the remedy text it was given:

```
doctrine worktree gc --fork dispatch/sl230-p01 --superseded-head 045e754cb970…
```

…which deleted the fork, after which `dispatch_reap` advanced the row on the
fork-absent path (I4) and reported `Reaped`. **Net effect: the D9 guard fired,
and its own remedy text walked the operator around it via the CLI escape.** The
commit beyond the imported tip was destroyed — the exact outcome RV-308 F-1
exists to prevent.

## Cause

D10 single-sources the CLI's remedy text and carries it **verbatim** into
`Refused.detail` (STD-001, and the ISS-246 fix). The text was authored for CLI
`worktree gc` callers, where "if this fork is funnel-managed, reap it with
`dispatch_reap`" is correct and useful routing. Carried unchanged into
`dispatch_reap`'s own refusal it inverts: it names the verb the caller already
used, and leaves only the override paths.

Verbatim single-sourcing is right for the *tokens*; it is wrong for a remedy
whose correct content is **caller-relative**.

## Fix direction

The remedy needs a per-caller arm while the reason token stays single-sourced.
For the funnel arm the honest remedy is neither `--force` nor
`--superseded-head` but: *the fork has advanced past what was imported — import
the new tip (`dispatch_import`) and re-verify, or abandon the extra commits
deliberately.* Also worth pinning by test: no refusal's `detail` may name the
verb that produced it.

## Governance touch points

- Confirms **NEW-OQ-C**'s own stated trigger to revisit the CLI `worktree gc`
  deferral: "sooner if an operator is observed reaching for CLI gc on a
  funnel-managed fork." Observed here on the first blind run — and not by
  accident, but *because the funnel's own refusal sent them there*.
- Sibling of ISS-246 (which asked for the CLI diagnosis to reach the MCP
  caller); this is the cost of having delivered that verbatim.
- Bears on D10's "a refusal's text **is** the recovery procedure" (FR-009).
