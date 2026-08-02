`doctrine ... reseat` renumbers an entity id and sweeps every textual reference
to it across authored source. The sweep is mechanical and it works. What it
cannot do is notice that a reference was **already aimed at the wrong record
before the sweep ran** — it rewrites `DEC-099` to `DEC-105` and leaves the
citation pointing at the same, still-wrong, record under a new number.

The failure mode is that the result *looks repaired*. The old id is genuinely
gone from the file, so a search for it returns zero hits and a reviewer reading
the reseat commit sees a correct, complete migration.

## The instance

SL-233 PHASE-15 shipped `src/commands/design.rs:1496`:

    // DEC-105 as revised (D7-R): re-baseline only to bytes demonstrably on disk.

`DEC-105` reads `status = "superseded"`, `superseded_by = ["DEC-100"]`. It was
already superseded at PHASE-15's own tip — the phase that wrote the comment
superseded the record the comment points at. The later reseat renumbered
`DEC-099` → `DEC-105` and carried the staleness forward intact.

A reviewer following that citation lands on a superseded record describing a
writer-lock-plus-compare-and-swap mechanism that **did not land**, while the
tolerance that did land is recorded only in `DEC-100`.

## How to check

Do not check that the old id is gone. Check that the id each reference names is
`accepted`:

    rg -n 'DEC-[0-9]{3}' src/ | while read -r hit; do :; done   # collect the cited ids
    doctrine knowledge show DEC-NNN                             # then read status on each

`rg -c <old-id>` returning 0 is evidence the sweep ran, and nothing more.

## Why it is easy to miss twice

A review pass found this and then dismissed it in its own write-up as "repaired
at HEAD", having read the reseat commit as a deletion of the line. **A renumber
is not a retarget.** The settling move is to read current state — the cited
record's status today — rather than to reason about what a commit did.

See [[mem.pattern.doctrine.conventions]] for the citation form itself, and
[[mem.pattern.harness.grep-negative-needs-positive-control]] for the neighbouring
discipline: a zero-hit search is a claim about the search, not about the code.
