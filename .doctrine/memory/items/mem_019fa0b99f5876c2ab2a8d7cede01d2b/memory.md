# Source the close_target from the admitted review candidate, not review/<slice>

When an audit repair lands **on the candidate branch** rather than through the
dispatch journal, `review/<slice>` does not carry it. The canonical close recipe
([[mem.pattern.dispatch.close-lands-via-candidate-integrate-trunk]] step 1) says
`--source refs/heads/review/<slice>` — follow it literally and trunk gets the
slice **without the repair, silently and green**. Nothing refuses: the merge is
clean, the gate passes, the tree-true checks pass. Only the RV's synthesis knows.

**The fix is one flag, not a recovery procedure:**

```
dispatch candidate create --slice N --label close-001 --role close_target \
  --payload code --base refs/heads/main \
  --source refs/heads/candidate/N/<review-label>
```

`--source` takes **any ref**. Point it at the ref the RV actually reviewed — the
admitted `review_surface` tip from `candidates.toml`
(`[current_admission.review_surface].admitted_oid`) — and the repair is in the
close_target by construction. The provenance gate is satisfied: `admit` checks
`parents == base + source`, and the recorded source *is* the candidate ref.

**Why this beats the two recorded alternatives.** Both existing routes solve the
same trap with more machinery and more risk:

- [[mem_019ee36939ca7a70b8aa960cb478d94c]] — cherry-pick the fix back onto
  `dispatch/<slice>`, `git branch -D review/<slice>`, re-prepare. Deletes and
  rebuilds an evidence ref.
- [[mem_019f06a18bf97b23bf771740e427b639]] — `git branch -f main <candidate-tip>`
  to pre-load trunk, then let a content-no-op close_target absorb it. Force-moves
  **trunk**.

This route touches no evidence ref and no trunk ref. It is also not exotic: it is
what `cand-227-close-001` did (`source_ref = refs/heads/candidate/227/fix-001`)
and what `cand-230-close-001` did (`source_ref = refs/heads/candidate/230/review-001`).
Two occurrences, both clean.

**Prerequisite — read the RV before choosing `--source`.** SL-230's RV-313 stated
it outright as the highest-value line in the ledger: *source from
`candidate/230/review-001` (18fc99613), never from `review/230` (66d478cc2), which
does not carry the F-3 fix.* `candidate status --slice N` shows both tips; when the
admitted `review_surface` OID differs from the `review/<slice>` tip, the delta is a
repair and `review/<slice>` is the wrong source. Check that inequality every close.

Unchanged from the canonical recipe: `admit --role close_target`, then the single
committing `sync --integrate --trunk`, then the ISS-030 tree-true checks.
