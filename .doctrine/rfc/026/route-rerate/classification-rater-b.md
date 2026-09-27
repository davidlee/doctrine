| ledger | F-N | severity | route-or-none | second-label | reason |
|---|---:|---|---|---|---|
| RV-366 | F-1 | major | owner-fix | stale | Design text lags the landed three-cell implementation. |
| RV-366 | F-2 | major | owner-fix | stale | Design’s general claim lags evidence found on another axis. |
| RV-366 | F-3 | major | owner-fix | stale | Code-impact table lags the changed files. |
| RV-366 | F-4 | major | owner-fix | neither | The finding concerns undeclared selector coverage and conformance semantics. |
| RV-366 | F-6 | major | owner-fix | duplicate | The decision has a live body but lacks its structured facet and links. |
| RV-369 | F-1 | blocker | demonstrate |  | The real terminal connection must work end to end. |
| RV-369 | F-2 | major | control |  | A test must reject the concrete broken terminal-feed implementation. |
| RV-369 | F-6 | blocker | review |  | Accepting image-size limits and their consequences is a design judgment. |
| RV-369 | F-7 | major | none | neither | Can the terminal’s image rejection be detected and reported? |
| RV-372 | F-1 | major | owner-fix | duplicate | Concurrent branches minted two live items with the same id. |
| RV-372 | F-3 | major | owner-fix | stale | The Facets comment lags the actual body-reading behavior. |
| RV-372 | F-4 | major | review |  | The measured savings and their cause challenge the product tradeoff. |
| RV-372 | F-5 | major | review |  | The finding asks whether to accept a negative human product verdict. |
| RV-372 | F-6 | major | review |  | The finding asks whether withholding populated fields without explanation is acceptable. |
| RV-372 | F-13 | major | none | neither | Can the handed-back worktree be independently gated in its environment? |
| RV-372 | F-14 | blocker | probe |  | The environment-variable hazard needs a hostile reproduction and safety check. |
| RV-372 | F-15 | major | owner-fix | stale | Other authored notes lag the changed command behavior. |
| RV-373 | F-1 | major | owner-fix | stale | Shipped text applies the changed rule beyond its intended scope. |
| RV-381 | F-5 | major | review |  | The finding asks whether governance gaps and draft policy are acceptable. |
| RV-381 | F-6 | major | control |  | A negative control must show the formatter rejects incorrect pointer rendering. |
| RV-387 | F-4 | major | control |  | The gate must reject the concrete failing reserve check under the ambient variable. |
| RV-390 | F-7 | major | control |  | A check must reject the synthetic run substituted for the required real run. |
| RV-395 | F-1 | major | control |  | A negative control must reject citations absent from the delivered set. |
| RV-395 | F-2 | major | owner-fix | neither | The finding asks whether implementation crossed the authored design boundary. |
| RV-395 | F-13 | major | control |  | The gate must reject stale shipped masters after edits without rematerialisation. |
| RV-317 | F-1 | blocker | probe |  | Hostile rendered fields need an adversarial escaping probe. |
| RV-317 | F-2 | major | probe |  | A malformed UTF-8 uid must be probed without crashing. |
| RV-317 | F-3 | major | control |  | A negative control must make discarded diagnostics visible as a test failure. |
| RV-321 | F-1 | blocker | probe |  | Persisted payload strings need an adversarial bounds probe. |
| RV-321 | F-2 | blocker | owner-fix | duplicate | Encoder framing and copied size literals give competing accounts of one bound. |
| RV-321 | F-3 | major | control |  | A constructor-bypass mutant must be rejected by the test. |
| RV-324 | F-1 | blocker | demonstrate |  | Accepted proposals must exercise the shell’s required effect protocol. |
| RV-324 | F-2 | blocker | probe |  | A concurrent human edit must be probed across the write window. |
| RV-324 | F-3 | major | control |  | A noncanonical citation must be rejected by a negative control. |
| RV-324 | F-4 | major | owner-fix | neither | The finding asks whether provenance retains a source fingerprint. |
| RV-342 | F-1 | major | control |  | A mismatched construction kind must make the agreement check fail. |
| RV-342 | F-4 | major | probe |  | The unrecoverable crash window needs a hostile interruption probe. |
| RV-380 | F-1 | major | control |  | A hand-set handler field must survive the canonicality check. |
| RV-380 | F-2 | major | demonstrate |  | The TS-to-Rust wire must be exercised through a real round trip. |
| RV-389 | F-8 | major | control |  | The parser-rejection mutant must fail against a stored snapshot fixture. |
| RV-389 | F-9 | major | demonstrate |  | The creation judgement must appear in the emitted change row. |
| RV-389 | F-13 | major | control |  | The stored-snapshot parser mutant must be rejected. |
| RV-389 | F-14 | major | control |  | Suppressing the sufficiency invalidation row must fail the agreement check. |
| RV-389 | F-16 | major | probe |  | Delegated null handling needs a hostile proposal-path probe. |
| RV-389 | F-17 | major | control |  | A shipped-surface internal id must be caught by a negative control. |
| RV-392 | F-1 | major | control |  | A mismatched directory and snapshot slice must be rejected. |
| RV-392 | F-3 | major | control |  | A title-only malformed record must be rejected as unreadable. |

**Counts:** review 4; demonstrate 4; probe 6; control 15; owner-fix 14; none 4. The second-label counts are duplicate 2, stale 7, neither 9. Total: 47.

**Route hesitation:** 7 — RV-366 F-4 (control / owner-fix); RV-369 F-1 (demonstrate / probe); RV-372 F-14 (probe / control); RV-381 F-6 (control / demonstrate); RV-321 F-2 (owner-fix / control); RV-324 F-2 (probe / control); RV-389 F-9 (demonstrate / control). Near `none` but assigned a route: 5 — RV-366 F-4 (owner-fix), RV-372 F-3 (owner-fix), RV-395 F-2 (owner-fix), RV-324 F-4 (owner-fix), RV-389 F-9 (demonstrate).

**Caveats:** No requested ledger had a severe-finding count mismatch; the total is 47. No finding contained a `route` field.