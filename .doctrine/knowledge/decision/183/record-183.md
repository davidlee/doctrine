# DEC-183: I10 quantifies over the subject-kind axis only

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What EN-2 asked, and what defining it turned up

`SL-249`'s `PHASE-02/EN-2` requires `I10`'s cell semantics to be defined per wire
key before the matrix is generated — design § 10 press item 1 warns the hazard is
that *"some keys are effectful only in combination"*, and that a cell asserting
the wrong side of the disjunction is a green test over a wrong mapping.

The key → honouring-kind mapping is clean. Each `Declaration` wire key is
consumed by exactly one arm of `declare` (`src/design_run/run.rs:1122`):

| honouring kind | keys |
|---|---|
| `inq-` | `question` `needs` `parent` `provenance` `lifecycle` |
| `sec-` | `body` |
| `att-` | `attests` `reviewer` |
| `fnd-` | `concerns` `summary` `blocking` `resolution` |
| `cp-` | `disposes` `dispose` |

`subject` is the addressing key and is honoured at every declarable kind.
`resolved_record` carries `#[serde(skip)]` and is not a wire key at all, which is
what `PHASE-02/EX-3` means by excluded *by construction*.

**Four keys are read on only one of their honouring kind's two paths**, verified
by reading each arm:

- `provenance` — only where `declare_node` creates (`run.rs:1158`). On an existing
  node `rebuild` carries `existing.provenance()` forward and the declared value is
  dropped.
- `lifecycle` — only where `declare_node` updates (`run.rs:1249`). The create
  branch returns before reading it.
- `concerns` and `blocking` — only where `declare_finding` raises (`run.rs:1419`,
  `run.rs:1429`). The dispose branch writes `resolution` and `summary` only.

So `provenance` on an `inq-` subject the run already holds is *silently accepted*
— the third state `I10` says does not exist, occurring at the **honouring** kind
rather than at a foreign one. `I10` as written in design § 5.5 is therefore
falsified by current behaviour, not merely undefined.

## The ruling

A cell is read **existentially over submissions**: key `k` is *observably
effectful* at subject kind `K` when some submission at `K` carrying `k` changes
the resulting snapshot observably; and `k` must be *refused* at every kind other
than its honouring one. The generated matrix proves the **kind** axis. It proves
nothing about the state axis, and must not be written so that it appears to.

This is deliberately the weaker of the two available readings. The stronger one —
that *every* submission carrying `k` at `K` is effectful or refused — is the one
`I10`'s prose states, and making it true is a behaviour change to four honoured
cells that no `PHASE-02` criterion authorises.

## What this leaves owed

- `ISS-327` — the state axis: the four cells above.
- `ISS-328` — an unknown key nested inside `CreateRecord` is dropped rather than
  refused, and `ISS-318`'s observation 3 asserts the opposite.
- Design § 5.5's `I10` wording is owed a narrowing correction at reconcile. The
  fix is prose, not code: `I10` should say what its generator proves.

The honest counter-argument, recorded rather than buried: narrowing here is the
same move `ISS-318` was raised for — fixing a class at the instances someone
noticed. What distinguishes it is that the residue is named and carried rather
than left silent, and that `PHASE-02` still closes the whole of the `SL-248` loss,
which is kind-inert throughout.

Related: `ISS-318` (the class), `RV-349` `F-4` (the inventory-versus-mapping
finding this is one level up from), `SL-249` `PHASE-02/VA-1` (which requires the
semantics be recorded in the test module, not only in the disposable phase sheet).
