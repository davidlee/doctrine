A text canary that checks a governance claim ("every record kind is enumerated
here") is only as good as the *measure* it uses for "enumerated". Pick the
measure by scoring the **pre-fix bytes** with it, not by reading the post-fix
corpus — the post-fix corpus satisfies every candidate measure, which is exactly
why it cannot discriminate between them.

## The concrete case (SL-249 PHASE-07)

`SPEC-019`/`PRD-010` had to name seven record kinds in paired form,
`assumption (ASM)`. Two candidate measures:

* **co-presence** — the long name appears somewhere in the tier AND the prefix
  appears somewhere in the tier;
* **strict adjacency** — `(?i:\blong\b)\s*\(\s*` + backtick? + `PREFIX` + backtick? + `\s*\)`.

On the corrected corpus both score 28/28. On the pre-amendment bytes co-presence
scored `spec-019.toml` at 4/7 and strict adjacency scored it at **0/7** — the
real answer, because that tier carried the long names in one parenthetical list
and the prefixes in a second, separate list. Co-presence would have shipped a
canary structurally blind to the defect it exists to catch, and would have
reported the corpus as two-thirds compliant when it was zero.

## The generalisation

The failure is not "we picked a bad regex". It is that a measure was validated
against data that could not falsify it. Two consequences worth carrying:

1. **Score the pre-fix bytes.** Recover them with `git show <sha>:<path>` and
   run the candidate measure over them. A measure that cannot separate the
   defect from the fix is not a measure.
2. **Freeze that scoring as the control.** A canary written after the corpus was
   corrected passes on its first run and proves nothing. Committing the pre-fix
   bytes as a fixture, and asserting the **exact** failure set the canary would
   have reported (not merely "non-empty"), is the only durable evidence that it
   can see anything at all.

The same shape applies to any conformance check authored alongside the fix it
would have caught: lint rules, schema validators, doc-coverage gates.
