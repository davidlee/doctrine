# DEC-132: Capsule credential refusal asserts on the file, via EROFS

## The sentence to carry forward

> **The credential refusal is `EROFS` from the mount flag, not `EACCES` from
> mode bits.**

The capsule runs as `uid=1000` and **owns** `~/.claude/.credentials.json`. It
could `chmod` it. Permission bits are not a boundary against a process that owns
the file — a read-only **bind mount** is, because the refusal comes from the
filesystem, not from an ownership check.

Everything else here follows from that one distinction.

**The direct consequence, for whoever narrows this profile next:** any change
that puts the secret back on a **writable** mount is a real weakening no matter
how restrictive its permissions look. `go-no-go.md` § 5 item 4 — scoping the
agent home's write access — points straight at that direction, which is why this
is a record and not a comment in the profile.

## Why the probe had to change

[[DEC-131]] made `$HOME` a tmpfs. That invalidated the old `api-cred` row, which
had been asserting on a read-only **directory** — a proxy for the property, not
the property.

F-P06-8 caught it, and STOP-2 held: a failed probe is a **finding and a
consult**, never a quiet rig edit.

The realigned row:

1. shows the credential **readable** first;
2. asserts refusal on **append, truncate, and unlink**;
3. runs a **positive control writing successfully beside it.**

Leg 3 is not ceremony. The home is writable by design, so without a
demonstrated successful write a broken write mechanism is indistinguishable from
an enforced boundary. Leg 2's `unlink` is the one a writable directory actually
requires — without it, replacement-by-removal was simply unasserted.

The result is **strictly stronger** than what it replaced: the old leg could
pass while the credential was writable.

The superseded rows stay in `evidence/results-c2.tsv`. Only the block under the
last `p-c2:` preamble is scored; the earlier `api-cred` rows are kept because the
realignment is only legible against what it replaced.

## The general lesson

A probe asserting on a **proxy** can pass while the property it stands for is
false. Assert on the property, and pair every refusal with a positive control.

---

# SL-241 sheet-decision index — the remaining 22

Twenty-seven decisions were minted in SL-241's runtime phase sheets. Five were
lifted to durable records; the other **22 stay indexed here**, resolving into
the archived sheets under `.doctrine/rfc/025/evidence/phase-sheets/`. This is a
**decision, not an omission**: their reach is inside SL-241 — rig construction,
probe scoring boundaries, sheet conventions — and a durable record for each
would cost more attention than it returns.

**Lifted:** [[DEC-128]] (D-P06-9) · [[DEC-129]] (D-P05-17) · [[DEC-130]]
(D-P06-6) · [[DEC-131]] (D-P06-5) · DEC-132 (D-P06-8, this record).

| id | what it settled |
|---|---|
| D-P05-6 | every sandbox-injected status is named; `*)` means the command's own |
| D-P05-7 | the heavy capsule builds its assets on site |
| D-P05-8 | H2 re-derived in § 5.6 as **dissolved** (operator-ruled) |
| D-P05-9 | F1 gains a `.doctrine/` design-target selector so H5 can reach it |
| D-P05-10 | guards (b) and (c) become their own T6 probes; H5's plant scoped |
| D-P05-11 | H12 plants its evaluation surfaces under the slice's own paths |
| D-P05-12 | H15 interrupts three stages; stage 4's indivisibility asserted apart |
| D-P05-13 | a tool the rig cannot invoke is a **defect of the rig** |
| D-P05-14 | egress is allowlisted, not binary, and per capsule kind — placement settled later by [[DEC-129]] |
| D-P05-15 | H11 scores at a different boundary per fixture |
| D-P05-16 | H7's disk bound is 20G (operator-ruled) |
| D-P05-18 | the 20G bounds the blast radius, **not** the cap H7 crosses |
| D-P05-19 | the capsule-time seam is a declarative per-cell lookup |
| D-P05-20 | the guard probes get their own executable and results table |
| D-P05-21 | guard (a) is a mechanical citation of H8, not a fifth script |
| D-P05-22 | guard (e) is three legs, and the baseline is one of them |
| D-P05-23 | `fx_case` lifted to `falsify-lib.sh`, as T5 said it would be |
| D-P06-1 | **EX-8 is discharged by its intent, not its letter** (operator ruling) |
| D-P06-2 | one scored attempt; prior attempts disclosed with their usage |
| D-P06-3 | the token accounting is reported in full, with its caveats |
| D-P06-4 | a rig defect is not an "attempt", and the boundary is stated |
| D-P06-7 | P-C2 **is** re-run against the changed profile |

**One of these has a live tail.** D-P06-1 is the ruling behind RV-343 F-5 —
PHASE-06 EX-8's letter is unmet and `plan.toml` does not record that. `plan.toml`
criteria are immutable-append and off the reconcile write surface, so the
one-sentence amendment (following PHASE-01 EX-7/EX-8's own precedent of a
trailing parenthetical) is the operator's call at `/close`. See `go-no-go.md`
§ 3.3 and [[DEC-128]]'s sibling reasoning on rulings that look like gaps.
