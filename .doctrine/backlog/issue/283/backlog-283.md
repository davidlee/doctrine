# ISS-283: supersede leaves the predecessor's status unmoved and validate passes it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found while adjudicating SL-233 RV-324 F-2, which needed DEC-105 to supersede
accepted DEC-092.

## Observed

```
$ doctrine supersede DEC-105 DEC-092
DEC-105 supersedes DEC-092
```

That correctly wrote both relation edges — `supersedes = ["DEC-092"]` on the
successor, `superseded_by = ["DEC-105"]` on the predecessor. It did **not** move
DEC-092's `status`, which stayed `accepted`.

```
$ doctrine validate
validate: corpus clean
```

So the corpus reported clean while holding **two `accepted` decision records
describing the same rule**, one of them explicitly `superseded_by` the other.
`superseded` is a valid status in the vocabulary — the setter's own error names it:

```
$ doctrine knowledge status DEC-092 bogus-state
Error: `bogus-state` is not a decision status
       (known: proposed, accepted, rejected, superseded)
```

The fix was a second, separate call: `doctrine knowledge status DEC-092
superseded`.

## Why it matters

Nothing surfaces the contradiction. It was caught only because the commit diff was
read line by line; an agent that trusts `corpus clean` leaves canon asserting both
records, and a later reader who reaches the predecessor first is misled by a record
the corpus still calls accepted. The failure is silent, and the surface that exists
to catch exactly this class of untruth passes it.

The blast radius is any consumer that filters by status rather than by relation —
`knowledge list --status accepted`, boot-snapshot rollups, retrieval ranking — all
of which will keep serving a decision that has been replaced.

## Two candidate fixes, not mutually exclusive

1. **`supersede` moves the predecessor's status** as part of the same verb. It
   already owns both records' `[relationships]` fields, so it is the natural home,
   and the two-step dance has no case where you want the edge without the status.
   Needs a decision on the non-knowledge kinds `supersede` also serves (ADR, and
   whatever else accepts the verb) — each has its own status vocabulary and may or
   may not have a `superseded` member.
2. **`validate` flags the inconsistent state** — a record that is `superseded_by`
   something while still `accepted` is a corpus defect, and the check is cheap and
   purely relational. Worth having even with (1), because hand-edited TOML can
   reach the same state without going through the verb.

(2) is the safety net and is kind-agnostic; (1) removes the footgun for the common
path. Preference is for both, (2) first, since it converts a silent wrong answer
into a loud one regardless of how the state was reached.

## Related

- `mem.fact.revision.no-knowledge-record-target` — the surrounding route (a
  Revision cannot target a DEC; correcting an accepted one is `knowledge new` +
  `supersede` + the separate status move). This issue is the sharp edge that
  memory has to warn about; resolving it lets step 4 of that memory be deleted.
- Captured live as a friction observation
  (`.doctrine/observations/records/c3/019fb180-0281-7ed0-8fba-1405d8ec11c3.toml`).
- Worked example in the corpus: DEC-092 / DEC-105, SL-233.
