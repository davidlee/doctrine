# Research Brief: What are the RV-374 / RV-382 / RV-377 design findings about, and which targets are repairs of earlier findings?

## Answer

The three ledgers are adversarial design reviews of three consecutive design-run changes. **SL-261 (RV-374, "Design adopt verb")** replaces the unusable `adopt_authored` payload protocol on `design apply` with a `design adopt`/`--diff` verb in which the engine derives and the caller confirms re-adoption of a hand-edited `design.md`. **SL-262 (RV-382, "Envelope names the next move")** replaces the turn envelope's `next_obligation` with a `forward` section that names the next move — unmet conditions, their discharging remedy, and a literal, applicable `ready` payload. **SL-263 (RV-377, "Ambient memory surfacing for pi and codex")** ports SL-205's ambient memory surfacing (retrieve → admit → dedup → cap → format) from Claude to the pi and codex harnesses via a neutral surface contract plus per-harness adapters.

All 18 blocker/major findings are overwhelmingly **B (mechanism prediction)**: the design asserted call chains, types, data shapes, file locations or reachability that the reviewer falsified against source in the tree (`design.rs`, `gate.rs`, `document.rs`, `retrieve.rs`, `boot.rs`, `submission.rs`, Node's `child_process`). Three are **A (costly surface)** because their diagnosis is a public commitment rather than a code detail: RV-374 F-2 (the `materialise` byte-identical promise), RV-382 F-1 (the envelope's no-drop bound, tied to DEC-293/REQ-437), and RV-382 F-4 (the `ready` payload agents are told to apply). No finding is C, D or E.

**Repairs-of-repairs (R)** exist only in RV-377, and only in the rounds that re-read previously repaired text: **F-9** (targets the §5.7 sketch F-2 rewrote to async), **F-16** (targets that same repaired sketch's `input` usage), and **F-22** (targets F-16's own `child.stdin.end` repair). RV-374's F-6 was raised after the F-1..F-5 integration but I verified via `git show` that it targets the *unmodified original* sec-2 state diagram, not repair text — so it is not R. RV-382 has one round only, so no R findings.

## Evidence

**Ledger shape.** `doctrine review status` / `review show` and the authored ledgers:
- RV-374 → SL-261, facet design, concluded, 6 findings, 21 rounds. Lock commit `f024b4cf3` ("lock design; slice to plan"); `git show f024b4cf3:.doctrine/slice/261/design.md | wc -l` = **576**.
- RV-382 → SL-262, design, concluded, 5 findings, 18 rounds. Lock commit `f8a13cd41` ("lock design after RV-382 — slice → plan"); design.md = **672** lines.
- RV-377 → SL-263, design, concluded (derived `active · await=raiser`), 23 findings, 68 rounds. Lock commit `b9bce77ee` ("advance the slice to plan (design locked at rev 41)"); design.md = **807** lines.
- Contests: the `rounds`/`contests` counters are the legacy baton counters (`.doctrine/state/review/<n>/baton.toml`; `src/review_ledger/derive.rs:179-201`) — **1 / 0 / 0** for 374 / 382 / 377. RV-374's contest is F-5 ("Contest accepted", F-5 response). No authored ledger has `[[finding.turn]]` rows, so contests cannot be recomputed from the ledgers.

**Severity populations.** RV-374: 1 blocker, 1 major, 4 minor, 0 nit. RV-382: 1 blocker, 4 major, 0 minor, 0 nit. RV-377: 0 blocker, 11 major, 11 minor, 1 nit. The 18 = RV-374 F-1,F-2 + RV-382 F-1..F-5 + RV-377 F-1,F-2,F-5,F-6,F-7,F-8,F-9,F-10,F-11,F-16,F-22.

**Round structure (R provenance).** `git log --oneline -- .doctrine/review/377/review-377.toml` and the integration commit bodies:
- `ddccae191` — "one in-session hostile pass (four findings …) and one independent pass (F-5..F-15 …)"; review-377.toml already holds 15 findings, design.md created (728 lines, rev 34).
- `e270e886b` — F-16..F-21 raised (21 findings).
- `ee6a0649f` — "integrate RV-377 F-22..F-23 (design rev 39)"; F-22 targets "The F-16 repair", F-23 targets "§5.1 now says …" (the F-18 repair).
- `1950d37d6` — "integrate the fresh adversarial pass (RV-377 F-16..F-21)"; F-16 "predates F-2's async switch, but the repair round kept it"; F-17 "The F-8 repair made §5.7 correct. But R-3's mitigation was extended"; F-18 "The F-6 repair joins relative paths onto cwd"; F-19 "With the new timeouts (pi `SURFACE_TIMEOUT_MS` … and the codex handler `timeout`)"; F-20 targets the "§7 adds three rows recorded as 'this design'" and "§9's phase-1 gate"; F-21 "Stale or unsupported prose left by the repair round".

**RV-374 F-6 is not R.** `git show 0636a5172` (F-1..F-5 integration) shows the sec-2 diagram lines unchanged (`Candidate --> RefusedMoved: pre-write re-check fails` / `Candidate --> Adopted: journal, snapshot written` as context); `git show 5035b3407` is the commit that introduces `Candidate --> Journalled`. The target diagram was original text.

**Classification source.** Each finding's `detail` names the wrong/unestablished code-level fact (class B) or the commitment (class A): e.g. RV-382 F-4 "Serializing `StageDeclaration` gives only `{"to":"..."}`, which cannot be applied as shown" (`submission.rs:894-905`, `commands/design.rs:1831-1833`); RV-377 F-16 "Node's asynchronous `execFile` … accept no `input` option … `node -e '…execFile("cat",[],{input:"HELLO"}…)'` prints `[] null`".

## Per-ledger summary

| ledger | slice | design lines | rounds | contests | findings (b/maj/min/nit) | A/B/C/D/E | R | L |
|---|---|---|---|---|---|---|---|---|
| RV-374 | SL-261 Design adopt verb | 576 | 21 | 1 | 1/1/4/0 | 1/1/0/0/0 | 0 | 0 |
| RV-382 | SL-262 Envelope names the next move | 672 | 18 | 0 | 1/4/0/0 | 2/3/0/0/0 | 0 | 3 |
| RV-377 | SL-263 Ambient memory surfacing for pi and codex | 807 | 68 | 0 | 0/11/11/1 | 0/11/0/0/0 | 3 | 8 |

## Classified findings (all 18)

| ledger | F-N | severity | class | R | L | B1/B2 | ≤12-word reason |
|---|---|---|---|---|---|---|---|
| RV-374 | F-1 | blocker | B | | | B2 | Two reads of design.md race; only an edit between reads exposes |
| RV-374 | F-2 | major | A | | | — | Narrows `materialise` byte-identical promise for accepted head bytes |
| RV-382 | F-1 | blocker | A | | L | — | Envelope no-drop bound / REQ-437 guarantee fails on large runs |
| RV-382 | F-2 | major | B | | L | B2 | `ready` derivation omits the authored-watermark refusal; needs an edited doc |
| RV-382 | F-3 | major | B | | | B2 | `Unmet` serde omits the discharging remedy; JSON consumers blind |
| RV-382 | F-4 | major | A | | L | — | `ready` payload cannot be applied as printed (agent-facing contract) |
| RV-382 | F-5 | major | B | | | B2 | Minted `ready` id collides only with a retained prior receipt |
| RV-377 | F-1 | major | B | | | B2 | `agent_id` gate inert on codex; subagent dedup parent-scoped (edge) |
| RV-377 | F-2 | major | B | | L | B2 | Sync spawn blocks pi's event loop; latency only |
| RV-377 | F-5 | major | B | | L | B2 | `additionalContextLimit` on the wrong codex object; canonicality blind |
| RV-377 | F-6 | major | B | | L | B2 | cwd-relative paths probed as root-relative; subdir cwd exposes |
| RV-377 | F-7 | major | B | | L | B2 | Per-probe loop re-implements the engine's multi-path query |
| RV-377 | F-8 | major | B | | L | B2 | Codex has no read tool; path surface never fires on reads |
| RV-377 | F-9 | major | B | R | L | B2 | Sketch lacks timeout/`ctx.signal`; a hung doctrine stalls (edge) |
| RV-377 | F-10 | major | B | | | B2 | Not unified-diff; `*** Move to:` header missing (rename input) |
| RV-377 | F-11 | major | B | | L | B2 | No captured codex payload; VTs self-authored; wire shape unestablished |
| RV-377 | F-16 | major | B | R | L | B1 | Async `execFile` silently ignores `input`; every pi fire empty |
| RV-377 | F-22 | major | B | R | | B2 | No stdin `'error'` listener; early-exiting child throws unhandled EPIPE |

## Totals across the 18

- **A = 3 (16.7%)**, **B = 15 (83.3%)**, **C = 0**, **D = 0**, **E = 0**.
- **R = 3 (16.7%)**: RV-377 F-9, F-16, F-22.
- **L = 11 (61.1%)**: RV-382 F-1,F-2,F-4; RV-377 F-2,F-5,F-6,F-7,F-8,F-9,F-11,F-16.
- **Class-B:** 15 findings — **B1 = 1** (RV-377 F-16), **B2 = 14**.

## Caveats

- **A vs B, classified A but defended as B by the written rule** ("data shape/signature wrong"): RV-374 F-2, RV-382 F-1, RV-382 F-4. Each diagnoses a public commitment *and* cites a code-level mismatch; I keyed on the commitment.
- **R provenance is ambiguous** for RV-377 F-9 and F-16: their target sketch was rewritten by the F-2 repair, but the specific defects (no timeout; the `input` option) predate it. I marked both R because the target text is repair text.
- **B vs D/C**: RV-377 F-8 ("decide deliberately whether to accept") and F-10 ("say whether the neutral wire should expose `class: patch`") carry a missing-decision flavour; I kept B because the reviewer supplied the facts and the agent resolved them without human input.
- **`contests` is a legacy baton counter**, not derivable from the authored ledgers (no `[[finding.turn]]` rows exist pre-SL-268). Values read from `.doctrine/state/review/<n>/baton.toml`.
- **Design-lock line counts** are `wc -l` at the lock commits (`f024b4cf3` / `f8a13cd41` / `b9bce77ee`); the current files are 587 / 679 / 807.
- The question's premise holds: exactly 18 blocker/major findings across the three design ledgers. No ledger was unreadable; the two forbidden artifacts were not read.
