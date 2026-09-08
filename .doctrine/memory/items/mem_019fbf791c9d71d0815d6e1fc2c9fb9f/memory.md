`doctrine slice conformance <slice> --against <range> --strict` counts a path as
declared **only when a `design-target` selector matches it**. A `scope-relevant`
selector does *not* clear the strict gate — the path is still reported under
`undeclared (n)` and the command still exits non-zero.

Observed directly on SL-241's own selector list (2026-08-02):

```
.doctrine/rfc/025/evidence/**   design-target   → .doctrine/rfc/025/evidence/x.md  CONFORMANT
.doctrine/knowledge/**          scope-relevant  → .doctrine/knowledge/x.md         UNDECLARED
```

**Why it bites.** `slice selector list` prints both intents in one table with no
hint that only one of them is load-bearing for `--strict`, so a slice that
"declares" a path by adding a `scope-relevant` selector reads as covered and
then refuses at the gate. The natural misdiagnosis is that the glob is wrong.

**How to apply.** When a path must pass `--strict`, give it a `design-target`
selector. Use `scope-relevant` for paths you want *associated* with the slice
(search, inspection, review scope) but do not intend to deliver. When a
conformance run reports `undeclared` for a path you believe is declared, check
the selector's INTENT before touching its glob.

Related: [[mem_019f239c569b75239987428d47b11f8f]] (a governance-output slice
reads as undeclared for the adjacent reason — nothing it produces is a
design-target).

## The fork this creates when you are told to touch a file outside the fence

Added at `SL-256` PHASE-01 (2026-09-08), where the mechanics above were
rediscovered from the other end: two files were edited to clear toolchain-bump
lint debt (`ISS-448`) that was reddening `doctrine check gate` before tests ran,
they were added as `scope-relevant` with an explanatory `--note`, and they still
reported `undeclared` at phase close.

The note was the right artefact reached for the wrong job. It **explains** a
widening to anyone reading `slice selector list`; it does not **declare** one.
Knowing that, there are two honest moves and one dishonest one:

- **Leave it `undeclared` and explain it** — phase sheet, commit message,
  backlog item. The cell is *working* when it surfaces a real widening, and a
  reader who finds the reason in three places is better served than one who
  finds a clean report. Usually right for a fix taken in passing.
- **Make it a genuine `design-target`** — correct only when the file really is
  one, i.e. the design should have named it and did not.
- **Re-intent it `design-target` to silence the cell** — buys a green report by
  asserting something false about what the slice set out to change. Don't.

**Caveat with an expiry.** `doctrine slice verify-vt` prints
`≈ UNATTRIBUTABLE VT-n — keyword present but <file> not modified by this slice`
on rows whose keywords it never checked — the clause is boilerplate in that arm.
Before a phase flips to `completed` there is no recorded delta, so *every* row
reports that way and none of it is evidence a mandate is satisfied. Confirmed by
before/after on an unchanged tree (`PHASE-02` `VT-4` went
`UNATTRIBUTABLE … keyword present` → `FAIL — keyword 'const READABLE' absent`
with no source change). Filed as `ISS-449`; delete this paragraph when it closes.
