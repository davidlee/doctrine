# CHR-082: Run and report routing trial two

The second `RFC-026` `P10` trial window, run under the protocol fixed by
`DEC-334` before it opens. Minted by `SL-270`'s design (inq-8); opens when
`SL-270` closes, as `CHR-077` followed `SL-260`.

## Owns

- Recording the **build floor** check (`DEC-334` item 2) at window open and at
  each eligible slice's design start. A failing check holds the slice.
- Recording each eligible slice as it is taken, and each waived one as skipped.
- Collecting the per-finding and per-ledger facts (`DEC-334` items 3–4).
- The blind second rater (item 5).
- The report, as `RFC-026` `E15`, read by item 6's rules and nothing added after
  the window opened.

## Does not own

- Any change to the route set, the lock check or the protocol. A change found
  necessary mid-window is recorded as a confound, not applied.
