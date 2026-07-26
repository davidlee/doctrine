# IMP-323: Handover must cite git/landing rituals by skill, never restate them

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The pattern

**Third occurrence in one night** (User's count, 2026-07-27): a `handover.md`
paraphrased a skill's git/landing procedure, the receiving agent followed the
paraphrase instead of the skill, and the paraphrase was missing a step.

The handover packet is *authoritative about state* — where the lifecycle stands,
which tree is canonical, what the last commit was, what not to re-derive. It is
**not** authoritative about procedure, but it reads as though it is: it is the
first thing the fresh agent loads, it is written by someone who just did the work,
and its imperative "Next actions" block looks exactly like an instruction to
follow verbatim. So the agent follows it and never opens the skill.

The failure is asymmetric and that is what makes it dangerous. A handover that
under-specifies *state* gets caught immediately — the agent can't proceed. A
handover that under-specifies *procedure* proceeds smoothly and destroys
something.

## The SL-228 instance (2026-07-27)

`handover.md` gave the close landing as:

```
doctrine dispatch refresh-base --slice 228
doctrine check gate                            # TWICE
doctrine slice verify-vt 228
doctrine dispatch sync --slice 228 --prepare-review
# only now remove the coord worktree dir (KEEP refs)
doctrine dispatch sync --slice 228 --integrate --trunk refs/heads/main
```

Accurate as far as it goes, internally consistent, and it names real verbs in a
plausible order — so nothing signals that anything is missing. But the `/close`
skill's step 3a mandates **creating and admitting a `close_target` candidate
before integrating**, and that step is absent. Following the handover, the agent
ran `prepare-review` with no admitted candidate, so the journal planned the
**code-only phase-chain tip** as the trunk payload; `--integrate` applied it as a
whole-tree fast-forward and deleted the entire `.doctrine/` corpus from `main`
(7659 files, 424548 deletions). Twice — because the second attempt admitted the
candidate *after* `prepare-review`, which is a silent no-op. See **ISS-255**.

Recovered fully (`git update-ref`), nothing lost, close halted. But the corpus was
gone from trunk for the length of two diagnoses, and only an unrelated merge
conflict made it visible at all.

## Why "just read the skill too" is not the fix

It is already the rule and it already failed. The handover's own framing defeats
it: a **"Next actions"** block with exact command lines is a strong signal that the
procedure question is settled. An agent that has just been told *"do not re-derive
RV-312's findings"* and *"the brief is already written"* reasonably reads the
whole packet as pre-digested. The economy of the handover — its whole value — is
that it saves you from re-reading things. That economy is correct for state and
actively harmful for procedure.

## Suggested change to `/handover`

Add a guard to the handover skill, and mirror it in the packet template:

1. **Never restate a skill's procedure.** For anything git-, landing-, or
   ritual-shaped (integrate, promote, worktree removal, ref surgery, close
   sequence), the packet **cites** — "`/close` step 3a governs the landing; run it
   as written" — and stops. It may add *state* the skill cannot know (which trunk,
   which refs are stale, what moved since the cut) but not reorder, abbreviate, or
   inline the steps.
2. **Mark any procedure block that survives as non-authoritative.** If a sequence
   really must be shown for orientation, it carries an explicit header:
   `ORIENTATION ONLY — <skill> is authoritative; read it before executing.`
3. **Authoring-time check.** `/handover` should ask directly: *does this packet
   contain command sequences that duplicate a skill's documented procedure?* If
   yes, replace with a citation.
4. Worth considering: have the packet template give "Next actions" a **routing**
   shape (`→ /close`) rather than a **command** shape, so the imperative pull goes
   to the skill instead of to a paraphrase of it.

## Acceptance sketch

- `/handover` skill carries the never-restate-procedure guard and the
  orientation-only marker convention.
- The `handover.md` template's "Next actions" section routes to skills rather than
  listing verbs, or carries the marker.
- Spot-check: a fresh agent handed a packet for a dispatched slice at close opens
  `/close` before running any landing verb.

Related: ISS-255 (the SL-228 corpus loss this caused), SL-228, `/handover`,
`/close`. Two earlier instances the same night were not individually filed — this
item covers the class.
