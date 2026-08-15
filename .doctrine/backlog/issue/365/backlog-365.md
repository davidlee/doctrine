# ISS-365: verify-vt's UNATTRIBUTABLE reason claims a keyword match it never ran

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`check_vt` (`src/vtgate.rs:97-130`) short-circuits on attribution at step (4) —
`!modified_files.contains(path)` — and returns `Unattributable` with the reason

```
keyword present but `{path}` not modified by this slice
```

The keyword loop is step (5) and has not run. So the *keyword present* half of
that sentence is unestablished, and where the keyword is in fact absent the
message is simply false.

Found while authoring `SL-251`'s plan. A VT mandate was pointed at
`src/design_run/attestation.rs` precisely *because* `fully_populated` does not
appear there today — making the keyword a real signal that the phase added the
attestation-side fixtures rather than an incumbent the gate would pass
vacuously. `verify-vt` reported the keyword present. Disproving the tool's own
claim cost a grep.

Two fixes, and the cheap one is probably right:

- **Drop the unverified clause.** ``` `{path}` not modified by this slice ```
  says everything the verdict actually knows, and it is the whole reason the
  row is unattributable.
- **Run the keywords first and report both facts.** Richer — it would
  distinguish *the signal is there but predates you* from *the signal is not
  there either* — at the cost of a file read step (4) currently avoids.

The misleading form is worst exactly where the gate is most useful: an author
choosing a keyword *because* it is absent today is doing the thing the mandate
wants, and is the reader most likely to be misled by being told otherwise.
