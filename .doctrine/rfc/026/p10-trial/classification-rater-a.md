# Research Brief: Design-review findings on RV-374 / RV-382 / RV-377, and repair-of-repair targets

## Answer

All 18 blocker/major findings on the three design-facet ledgers are **mechanism-prediction or costly-surface findings about the design's code-level claims**; none is a pure prose, citation, or missing-human-intent finding, and there is no `E`. Slightly under two-thirds (11/18) are class **B**: the design asserted a call chain, type, signature, data shape, reachability, or file location that was wrong or unestablished. Seven are class **A**: they turn on a persisted/generated schema, the agent-facing CLI output contract, or a binding governance requirement. The clearest split is that RV-374 and RV-382 majors are mostly about the *shape of what the verb emits or persists*, while RV-377 is almost entirely about *what the harness wires actually do*.

Three findings target text written to repair an earlier finding (R): `RV-382` F-5 (the minted `advance-<to>-r<rev>` id was introduced by F-4's fix and then found collidable); and `RV-377` F-16 (the async `execFile` sketch produced by F-2's fix) feeding `RV-377` F-22 (the `child.stdin.end(json)` write produced by F-16's fix). This is one explicit chain in each of RV-382 and RV-377; a longer implicit chain — F-2 → F-16 → F-22 — runs through the pi adapter sketch. Nine findings carry L, i.e. a later finding in the same ledger revisits the same mechanism or the repair text.

Confidence: high on the mechanics and the R chains (they are quoted in the finding/disposition text); lower on the A/B boundary, which I resolved with a stated rule and flag below.

## Ledger summaries

| ledger | slice | design lines at lock | rounds | contests | blocker/major/minor/nit | A/B/C/D/E | R | L |
|---|---|---|---|---|---|---|---|---|
| RV-374 | SL-261 (Design adopt verb) | 576 (`f024b4cf3`, "lock design") | 21 | 1 | 1 / 1 / 4 / 0 (6) | 1 / 1 / 0 / 0 / 0 | 0 | 0 |
| RV-382 | SL-262 (Envelope names the next move) | 672 (`f8a13cd41`, "lock design after RV-382") | 18 | 0 | 1 / 4 / 0 / 0 (5) | 4 / 1 / 0 / 0 / 0 | 1 | 1 |
| RV-377 | SL-263 (Ambient memory surfacing for pi and codex) | 807 (`b9bce77ee`, "design locked at rev 41") | 68 | 0 | 0 / 11 / 11 / 1 (23) | 2 / 9 / 0 / 0 / 0 | 2 | 8 |

## Classified findings

| ledger | F-N | sev | class | R | L | B1/B2 | ≤12-word reason |
|---|---|---|---|---|---|---|---|
| RV-374 | F-1 | blocker | B | | | B2 | Design's apply pipeline keeps two reads of design.md |
| RV-374 | F-2 | major | A | | | — | Byte-identical authored design.md promise vs parser-dropped head bytes |
| RV-382 | F-1 | blocker | A | | | — | No-drop envelope bound; contradicts REQ-437/DEC-293 |
| RV-382 | F-2 | major | B | | | B2 | Ready omits the run's authored-divergence refusal guard |
| RV-382 | F-3 | major | A | | | — | Promised JSON remedy absent from serialised envelope |
| RV-382 | F-4 | major | A | | L | — | Printed ready payload is not an applicable ApplyRequest |
| RV-382 | F-5 | major | A | R | | — | Minted submission_id collides with a retained receipt |
| RV-377 | F-1 | major | B | | | B2 | Codex PreToolUse has no agent_id; subagent dedup parent-scoped |
| RV-377 | F-2 | major | B | | L | B2 | Pi adapter sketch spawns synchronously on the event loop |
| RV-377 | F-5 | major | A | | L | — | Codex config puts additionalContextLimit on the wrong object |
| RV-377 | F-6 | major | B | | L | B2 | Patch/pi paths are cwd-relative but treated as root-relative |
| RV-377 | F-7 | major | B | | L | B2 | Per-probe fan-out reimplements the engine's multi-path query |
| RV-377 | F-8 | major | B | | L | B2 | Codex lacks Read; path surface reachability and timing wrong |
| RV-377 | F-9 | major | A | | L | — | No timeout on pi spawn or codex handler config |
| RV-377 | F-10 | major | B | | | B2 | Misnamed parser; omits codex's `*** Move to:` header |
| RV-377 | F-11 | major | B | | L | B2 | Codex wire shapes asserted, not captured; self-authored VTs |
| RV-377 | F-16 | major | B | R | L | B1 | Async execFile ignores `input`; doctrine gets empty stdin |
| RV-377 | F-22 | major | B | R | | B2 | Repaired stdin write has no error listener; EPIPE kills host |

## Totals

- Class A 7 (38.9%), B 11 (61.1%), C 0, D 0, E 0.
- Marked R: 3 (`RV-382` F-5; `RV-377` F-16, F-22).
- Marked L: 9 (`RV-382` F-4; `RV-377` F-2, F-5, F-6, F-7, F-8, F-9, F-11, F-16).
- Class-B split: B1 = 1 (`RV-377` F-16); B2 = 10.

## Evidence

- Population: `jq` over `.doctrine/review/{374,382,377}/review-*.toml` and `review show --json` gives exactly the stated severities; 2 + 5 + 11 = 18 blocker/major.
- Rounds/contests from the runtime batons: `.doctrine/state/review/374/baton.toml` (`rounds = 21`, `contests = 1`), `382/baton.toml` (`rounds = 18`, `contests = 0`), `377/baton.toml` (`rounds = 68`, `contests = 0`).
- Design-lock line counts (inferred lock identity from commit subjects, not from a lock API): `git show f024b4cf3:.doctrine/slice/261/design.md | wc -l` = 576; `f8a13cd41` = 672; `b9bce77ee` = 807.
- Repair chains quoted in the findings:
  - `RV-377` F-16: "This dates from the original sketch (it predates F-2's async switch), but the repair round kept it." (`.doctrine/review/377/review-377.toml:224`)
  - `RV-377` F-22: "The F-16 repair writes the request with `child.stdin.end(json)`, but attaches no `error` handler…" (`:300`)
  - `RV-382` F-5 details the `advance-<to>-r<revision>` mint; F-4's response introduces it ("Doctrine-minted submission_id advance-<to>-r<revision>"), and the pre-repair draft has no `advance-` string (`git show d1f5edd03:.doctrine/slice/262/design.md | grep advance-` empty).
- Explicit later-finding references establishing L: `RV-377` F-17 "The F-8 repair…" (`:236`), F-18 "The F-6 repair joins relative paths…" (`:248`), F-19 "the new timeouts" (`:264`), F-20 "engine probe arity… phase-1 capture gate" (`:274`), F-22 "The F-16 repair…" (`:300`).

## Judgement

**The A/B boundary is the real interpretive fork, and I chose subject-shape over defect-location.** My rule: A when the finding's *primary subject* is a persisted/generated format or schema, the agent-facing CLI output contract, or a governance seat; B when the subject is whether the implementation behaves as claimed. Under the looser reading ("the fix touches a public surface"), nearly all 18 would be A, because all three slices ship agent-facing surfaces — that reading collapses the taxonomy and cannot be what the classes are for. Under the stricter reading (B = a genuinely internal call/type/data error), the four `RV-382` envelope/ready findings are A, not B, because their dispute is over the *shape of what the design-run verb emits* — F-4 literally argues ready must be a complete `ApplyRequest`, F-3 that the JSON row must expose the remedy, F-5 that the minted identity must be reserved. These are surface-design commitments, not incidental bugs. `RV-382` F-2 is the opposite: its fix is a missing internal guard (`Forward.diverged` blocks ready), so B.

**`RV-377` F-5 and F-9 I read as A because the artefact at issue is generated, installed config** (codex's `hooks` schema; the `timeout` field and named constants). The response language confirms both are about what doctrine writes into an existing install and whether that write heals — F-5 even notes a hand-edited or pre-limit entry "is judged canonical and never healed", which is a persisted-format entrenchment. F-1 and F-6 look superficially similar (wire/behaviour deltas) but their subject is runtime decoding behaviour, so B.

**The L flag is where I was most deliberate.** I marked L when the later finding explicitly names the earlier repair ("F-N repair/wording") or unmistakably re-opens the same function. The explicit chain is F-2→F-16→F-22, F-6→F-18/F-23, F-8→F-17, F-9→F-19, F-11→F-20; F-20 also re-opens F-7 (engine probe arity). I marked `RV-377` F-5 L on the narrower ground that F-9 concerns the same codex handler-field mechanism and cites §5.6's `additionalContextLimit` argument — but the two were almost certainly raised in the same pass (F-5 references "the timeout finding"), so if L is meant to require a later *round*, F-5's L should go. I left F-10 blank for L even though F-20 touches "path resolution", because F-18/F-23 attribute the path-join to F-6, not F-10.

**`RV-377` F-16's R is the one I would most expect an equally careful reader to dispute.** Its target line *was* rewritten by F-2's fix (Sync → async), so it is a repair-of-a-repair in the literal sense; but the finding itself says the `input` defect "predates F-2's async switch". I mark R because the task defines R by the provenance of the target text, not the provenance of the bug.

**Class B1/B2:** I could only justify one B1. F-16 ("every pi fire … surfaces nothing") breaks the primary path, so a thin happy-path implementation exposes it. Everything else needs an unusual input, a threshold, a hung/early-exiting child, or a real harness rather than the design's synthetic fixtures — the design's own tests pass, which is exactly why these findings exist.

## Limits

- The A/B line is judgement, not extraction. Hesitations: `RV-382` F-1 (A/REQ-437 vs B/bound-mechanism), `RV-382` F-3 and F-4 (A/public payload vs B/B1 data-shape errors), `RV-374` F-2 (A/authored format vs B/parser mechanism), `RV-377` F-5/F-9 (A/generated config vs B/field-placement).
- `RV-377` F-16's R and B1, and `RV-377` F-5's L, are the flags most sensitive to interpretation; see Judgement.
- R/L infer round boundaries from commit messages, design-run receipts (`.doctrine/state/slice/*/design.toml`) and the findings' own line-number drift; the design run does not expose a per-finding round field.
- Design-lock line counts depend on identifying the lock commit: 261 `f024b4cf3` and 262 `f8a13cd41` name "lock", 263 `b9bce77ee` says "design locked at rev 41". If "at the lock" means the last pre-lock revision instead, 261 and 263 are unchanged (576/807), 262 unchanged (672) — so this is low-risk.
- I deliberately did not read `.doctrine/rfc/026/rfc-026.md` E12 or `/tmp/e12-draft.md`; nothing in the findings above required them.
- Premise check: "18 findings" holds exactly (2+5+11).
