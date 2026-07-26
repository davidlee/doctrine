# ISS-255: Integrate plans the code-only phase-chain tip as trunk payload; whole-tree FF deletes the corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Summary

`dispatch sync --prepare-review`, run when **no `close_target` candidate has been
admitted**, plans the trunk journal row against the **phase-chain tip**. The
`phase/<N>-NN` refs are the class-routed **code-only** projection (ADR-012) — their
trees contain **no `.doctrine/` at all**. When trunk is an ancestor of that chain,
`--integrate` applies the row as a **whole-tree fast-forward**, silently replacing
trunk's tree and deleting the entire authored corpus. No conflict, no refusal, no
diagnostic — the run reports `(advanced+pure-ref)` and `10 ref(s) replayed`.

Witnessed twice at the SL-228 close (2026-07-27), reproducibly:

```
integrate: refs/heads/main 139166dbf45d..d66180c25351 (advanced+pure-ref)
integrate: 10 ref(s) replayed
```

`139166dbf..d66180c25` = **7659 files changed, 15580 insertions, 424548 deletions**
— the whole of `.doctrine/` (every ADR, spec, slice, standard, policy, RFC, backlog
item, memory) removed, plus a deleted `todo.md` resurrected. Trunk was restored by
hand both times (`git update-ref refs/heads/main 139166dbf45d`); nothing was lost,
because the failure was caught before anything merged onto `edge`.

## Evidence — which ref carries what

```
review/228     .doctrine present = 1     ← correct trunk payload
phase/228-09   .doctrine present = 0     ← what the journal planned
phase/228-01   .doctrine present = 0

git diff --name-status main review/228 -- .doctrine  →  9 A, 5 M, 0 D
```

`review/228` **adds and modifies** corpus paths and deletes none. The phase chain
has no corpus to begin with. The journal row named the wrong one:

```toml
[[row]]
source_oid       = "d66180c25351…"   # phase/228-09 tip — code-only tree
target_ref       = "refs/heads/main"
expected_old_oid = "139166dbf45d…"
planned_new_oid  = "d66180c25351…"
applied_new_oid  = "d66180c25351…"
status           = "verified"
```

## The three distinct defects

1. **Wrong payload class chosen silently.** With no admitted `close_target`,
   `prepare-review` falls back to the phase chain for the *trunk* row. Every prior
   slice landed as `candidate(NNN/close-001): merge refs/heads/review/NNN` (SL-204,
   208, 224, 226, 227) — a merge whose first parent is trunk, which preserves the
   tree. The fallback should **refuse** ("no close_target admitted; trunk payload
   would be the code-only projection"), not silently pick a corpus-less tree.
2. **SL-166's corpus-clobber guard did not fire.** `--allow-corpus-clobber` is
   documented as fail-closed: *"Absent for a clobbered path ⇒ the advance is
   refused"*. 7659 authored `.doctrine/**` paths were deleted and nothing refused.
   Either the guard does not cover the pure-FF path (it may only inspect merge
   results), or it does not run on the `--trunk` leg. **This is the load-bearing
   defect** — the guard exists precisely for this and was the last line.
3. **Admitting a candidate after `prepare-review` is a no-op, silently.** The
   correct candidate was created and admitted between the two attempts
   (`cand-228-close-001` at `f898c2a3` — first parent `main`, second `review/228`,
   45-file code delta, corpus intact, verified by inspection). `--integrate` then
   *replayed the stale journal row anyway* and produced byte-identical wrong
   output. Nothing warned that the admission had no effect on an already-planned,
   already-`verified` row. Adjacent to IMP-304 (superseding a Failed/Pending row) —
   this is the **`verified`** case, which IMP-304 does not cover.

## Relationship to ISS-056 (closed) and SL-166 (done)

Same **outcome shape** as ISS-056 — "stale-base dispatch integrate silently deletes
authored corpus", closed `fixed`, guards built by SL-166 — but a **different root
cause**, which is why the guards missed it:

- **ISS-056**: the *fork base* predated the corpus, so the phase tree never had it.
- **ISS-255**: the fork base is fine; the phase refs are *by design* a code-only
  projection, and the bug is applying that projection to trunk as a whole-tree FF.

ISS-056's fix hardened against a corpus-less **base**. Nothing hardened against a
corpus-less **payload class**. The outcome is identical and equally catastrophic,
so the ISS-056 guards should be extended to cover it rather than a new mechanism
being invented.

## Aggravating factor: `refresh-base` makes the FF *possible*

The whole-tree FF can only happen when trunk is an **ancestor** of the phase chain.
`dispatch refresh-base` merges trunk into `dispatch/<N>`, so after the mandated
close-time refresh the re-cut phase refs descend from trunk and the FF path opens.
Before a refresh, integrate would have to merge (and merging a corpus-less tree
produces a conflict or a visible mass deletion). **So the close ritual's own
prescribed step is what arms this.** That ordering deserves a test.

## Suggested acceptance

- `prepare-review` refuses to plan a trunk row from a payload whose tree lacks
  `.doctrine/` when the current trunk has it — naming the admitted-candidate
  remedy.
- The SL-166 corpus-clobber guard is proven to run on the **pure-FF trunk advance**
  path, not only on merge results. Regression test: trunk with corpus + payload
  without ⇒ refused absent `--allow-corpus-clobber`.
- `candidate admit` warns (or refuses) when a trunk row for that slice is already
  `verified`/applied, so the admission's no-op is visible.

Related: ISS-056, ISS-038, SL-166, IMP-304, ADR-012, SPEC-022, SL-228 (found at close).
