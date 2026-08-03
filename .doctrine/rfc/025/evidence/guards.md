# RFC-025 · C3 ingestion probe — guard probes and the conflict sub-probe

Companion to `README.md` and `matrix.md`. Two things live here that are **not**
matrix rows and must never be summed with them.

## ⚠ Reading `results-c3.tsv`: four `fail` rows are four successes

The conflict sub-probe recorded its **falsification round in-band**. Four rows
carry `outcome=fail`, and each is stamped in its preamble:

```
p-c3: … legs=conflict rows=H10 MUTATED=m32-peer-not-from-b.sh
p-c3: … legs=conflict rows=H10 MUTATED=m33-halves-agree.sh
p-c3: … legs=conflict rows=H16 MUTATED=m34-trunk-never-moves.sh
p-c3: … legs=conflict rows=H16 MUTATED=m35-move-before-pinning.sh
```

A mutant that reds is the mutant **working**. Anything counting outcomes must
respect the `MUTATED=` stamp on the preceding `p-c3:` line; a naive count of
`outcome=fail` reports four failures where there were none. The guard probes
write to a **separate file** (`results-guards.tsv`) with its own `MUTATED=`
stamp, which is why the overlay variable is per-probe — `SPIKE_GUARDS_MUTANT`,
not `SPIKE_C3_MUTANT` (D-P05-20).

## The five guard probes

**Why they exist: a guard never seen to fire is not known to work** (EX-10,
VA-2). Each of the five is *observed refusing at least once*. They have no
`Hnn` trio, no fixture × mechanism cross-product and no altitude — so they are
not matrix rows, and they get their own executable (`control/probe-guards.sh`,
`rig guards [a…e]`) and their own results file (D-P05-20).

**Scored 9/9 `pass`, 71 assertions, 0 red. Falsified 4/4 first** (m36–m39).

| guard | fixture | leg | observed | what it proves |
|---|---|---|---|---|
| **(a)** | — | `cite:H8/gitlink` | 4 scored entries at `conform/gitlink` | F-2's mode-aware leg, **by citation** — H8 already observed it on both fixtures and both mechanisms |
| **(a)** | — | `cite:H8/gitmodules` | 4 scored entries at `conform/gitmodules` | as above; cited, not re-run, so the corpus holds one copy of the observation |
| **(b)** | light | `nonascii` | `conform/forbidden-path` | leg 3 fires on a non-ASCII `.doctrine/` path, **isolated in the range** |
| **(b)** | heavy | `nonascii` | `conform/forbidden-path` | same, on the heavy fixture |
| **(c)** | light | `rename-out` | `conform/forbidden-path` | leg 3 fires on a rename **out of** `.doctrine/`, isolated |
| **(d)** | light | `verify-shadow` | `verify/suite-failed` | a capsule-authored `verify.sh` changes nothing — the verdict is the ro-bound `/rig/verify.sh` running B's command |
| **(e)** | light | `baseline` | `no refusal` | the run the substitution legs are compared against |
| **(e)** | light-inrepo | `decl-worktree` | `no refusal` — **byte-identical to baseline** | the control plane never reads a declaration the capsule can write |
| **(e)** | light-inrepo | `decl-committed` | `conform/undeclared-path` | the S-side substitution never reaches the control plane's read either |

### What each one is careful about

**(a) is a citation, and it can fail** (D-P05-21). H8 *is* guard (a). The leg
counts H8's entries out of the committed `results.tsv`, with a negative control
on a token H8 never produces. A citation that cannot fail is prose.

**(b) does not prove what it looks like it proves.** It fires on a non-ASCII
path, but that does **not** show `core.quotePath=false` is load-bearing — `-z`
already defeats that evasion (F-P05-23). The isolation clause ("exactly one
governance path in the range") is **asserted, not assumed**: leg 3 returns on the
first match, so without the count guard, (b) is a differently-named re-run of H5
(F-P05-22). Mutant m36 proves it — the refusal and the ingestion stay green while
the observation quietly becomes about another path.

**(c) runs both directions.** With `--find-renames` the `.doctrine/` source leg
vanishes and the same capsule passes, so `--no-renames` is *shown* load-bearing
rather than asserted. **Light only** — heavy has no source to rename at B
(F-P05-21).

**(d) needed a genuinely broken suite to observe anything** (F-P05-42): the
honest `verify:` command passes, so a guard asserting "the verdict is unaffected"
is vacuous against a green suite. `audit-i4a.sh` is the **static** complement (no
runner copied in); this is the behavioural half and neither substitutes for the
other. Both ran.

**(e) is three legs** (D-P05-22), and it is **QUE-201's only evidence input** —
recorded as **EVD-011**. The trusted-side observable excludes the OIDs
deliberately; those are fixture identity, not behaviour.

> **Do not generalise (e)'s third leg's token — F-P05-43.** *Where* the committed
> rewrite refuses is **fixture-specific**: F2 keeps its declaration copy at the
> repository **root**, and SL-001 declares selectors for `src/**` and
> `.doctrine/**` only, so conform leg 2 refuses it as undeclared before leg 3 or
> anything later ever looks. A project that declared its own declaration path
> would get past leg 2, and the clause that would still hold is the one the leg
> actually asserts: **the control plane resolved B's command regardless.** The
> leg therefore records the refusal as an *observation* and asserts the
> provenance separately. A reader who takes `undeclared-path` here as "the model
> refuses declaration substitution at conform" has learned something false about
> every other project.

## The conflict sub-probe

H10 and H16 each owe a leg against the **real candidate layer** — the incumbent's
resolution path (§ 5.1 D8) — rather than against the probe pipeline. It runs on a
fourth fixture variant, `light-plan`, which carries a plan and phases because
`prepare-review`'s phase-completion gate is out of the pipeline (EX-15).

**Scored 2/2 `pass`, falsified 4/4. Altitude `counts-toward-nothing`** (VA-4) —
these legs are explicitly excluded from the model-level claim, because they
measure the *incumbent*, not the capsule model.

| leg | stage | token | planted |
|---|---|---|---|
| H10 | `candidate-create` | `conflicted` | `pair-meets src/capsule-stub.ts 018cbd1/14f0650` |
| H16 | `integrate` | `stale-trunk` | `trunk-moved f2b3885->22fdb32` |

**H10** — the pair is classified `Conflicted` and parked for hand-resolution.
**H16** — `create` and `admit` both accept a moved trunk; the fast-forward CAS at
`integrate` is the **sole** place staleness is caught.

### The finding this produced, and it is open

**F-P05-40 / ISS-305.** On the candidate layer the two refusals are *not the same
kind of thing*:

- a **conflict** refusal is **ledgered** — `candidate create` exits **zero**, and
  the verdict is `status="conflicted"`;
- a **staleness** refusal is **status-borne** — the verb exits non-zero.

**The clap help asserts the opposite.** Filed as ISS-305; `src/` was held
untouched this phase (S4). It is also an input QUE-202 owes — how the capsule
model *admits* a second result. Refusal is proven; admission is not designed.

## Re-running

`drivers/` holds T5's and T6's rounds, committed and re-runnable. `rig`'s parser
owns the flag space, so a probe-specific flag must go to the probe directly —
`./control/probe-guards.sh --positive-control`, not `rig guards
--positive-control` (F-P05-9).

T4a–T4e's drivers were never tracked and are gone (F-P05-39). Do not reconstruct
them from prose and re-run them under the old claims' names.
