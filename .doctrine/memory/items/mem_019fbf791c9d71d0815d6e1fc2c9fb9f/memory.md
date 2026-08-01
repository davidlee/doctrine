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
