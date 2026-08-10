# REV REV-051 — reconcile SL-248

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-248` (capsule provisioning and Linux backend) closed ten phases and settled
`RV-352`, its reconciliation audit. Two governance items survive into authored
spec truth. Both concern `SPEC-030` and its `REQ-459`.

## Row 1 — `REQ-459` `pending` → `in-progress`

`REQ-459` states that every platform backend enforces equivalent freshness,
filesystem, process, credential, network, resource and canonical-authority
properties, with Linux/bubblewrap the only measured initial backend.

`SL-248`'s honest reading of its three criteria (`RV-352`, `notes.md` item 139):

| criterion | state after `SL-248` |
|---|---|
| 1 — a shared conformance suite proves the seven property families | **discharged over the channels `sec-2`'s channel ledger names.** The qualifier is load-bearing: *in full* was claimed four times against four tables in this slice and was false the first three |
| 2 — Linux/bubblewrap may become the initial supported backend from `SL-241` evidence **and production acceptance tests** | **partial** — two shortfalls, below |
| 3 — no other backend claims support until it passes the same suite independently | **discharged structurally** — one suite parameterised by backend; a second backend passing it edits nothing |

`in-progress`, not `active`. `active` would assert the closure criterion 2 does
not support. `pending` no longer describes a requirement two of whose three
criteria are discharged.

### Criterion 2, first shortfall — there are no production acceptance tests

The criterion requires `SL-241` evidence **and** production acceptance tests.
`SL-248` delivered the conformance suite and no production acceptance tests. This
is the shortfall `RV-352`'s reconciliation brief already carried.

### Criterion 2, second shortfall — the suite's own capsules are permissive on a store host

**This shortfall is not in `RV-352`'s brief.** It was found after the audit
concluded, by discharging `ISS-339` — running `just capsule-verify` on a real
NixOS host for the first time. It is recorded here because a backlog item does
not discharge a reconciliation ruling.

The conformance fixture derives its readable-input set by binding **whole host
top-level roots**, `/nix` among them. `--ro-bind` is read-only, **not `noexec`**:
measured off-jail, a capsule with `/nix` bound sees **691 store paths** and runs
`git`, `curl` (TLS-capable) and `gcc`.

Production already refuses exactly this shape. `readable_set` binds only declared
`readable-roots` plus resolver-expanded `closure-roots`, and `PHASE-10` `EX-8`
states that no configuration can make a host-wide store readable whole — because
binding a store whole gives a capsule a larger executable set than an ordinary
unconfined process has, which inverts `REQ-459` property 2, an *explicit* input
set, into an implicit and enormous one.

So the fixture is the host-shaped default `EX-8` exists to rule out — and it is
the fixture that hosts the capsules in which `BoundedInputSet` is demonstrated.

**What this costs the record.** The nineteen conformance rows are worth less
off-jail than a reader of the verdict would assume: the capsules the properties
are demonstrated in are more permissive than the ones production builds. The rows
are not falsified — the properties they assert still hold as measured — but the
environment they were measured in does not match the environment production
constrains, and criterion 2 is about production.

Carried as `ISS-341`, whose design gets its own slice (`SL-252`) by the owner's
call: the fixture should declare `closure-roots` for its shell rather than a
top-level ancestor, and the tension to resolve is that a closure resolver is
host-shaped by nature while production escapes that by making it
operator-declared. A fixture has no operator and must not grow a Nix dependency
(`POL-002`).

`ISS-341` blocks `ISS-340` (the one remaining red row), and the ordering is
counter-intuitive on purpose: the cheap fix for `ISS-340` — filter roots to those
bearing an executable — turns the transcript green while leaving `/nix` bound
whole. A green transcript with the serious half unfixed is worse than the current
red, because the red row is presently the only thing pointing at `ISS-341`.

### What `ISS-339` bought, and it was not only bad news

The same off-jail run closed `sec-9` residual 3's neighbourhood **by
measurement**: off-jail the trusted side does not already hold `no_new_privs`, so
`sec-9`'s observation reads **caveat-free** and the backend's provenance for the
bit is established rather than argued. That is what `ISS-339` existed to do.

Three defects in three runs, all in the fixture, all invisible in the jail. The
pattern behind them is one thing worth carrying forward: **a derivation that is
accidentally correct in the single environment it has ever run in.** The jail
supplies `/bin` twice over by coincidence, which masked each defect in turn.

## Row 2 — `SPEC-030` must name the shipped suite's host dependencies

`setsid` (from `util-linux`) and `socat` are now **production host requirements
of the shipped conformance suite**, discovered by this audit the hard way.

- `setsid` is what makes row 7's process-tree teardown measurable at all: the
  row's payload forks a descendant that leaves the original session, precisely so
  a process-group-only backend cannot pass.
- `socat` carries both legs of row 5's network probe. It replaced `/dev/tcp`,
  which is a **bash extension** while `SHELL` is contracted only to be POSIX —
  a latent false `Unproven`. Without `socat` the row reads `Indeterminate` rather
  than silently passing.

Both are declared in `flake.nix` for this project's jail, which is a
repository-controlled file and not a spec. The spec should name them where the
suite's contract lives, so a host preparing to run the suite learns of them
before a red row teaches them.

The audit hit this concretely: a jail built from `edge`'s flake, where the change
had not landed, could not resolve `setsid`, and the suite went 7-red for a reason
that was not a conformance defect.

## Landing

Row 1 (`status`) auto-lands at `revision apply`. Row 2 (`modify`) is surfaced for
manual landing under the authored-truth honour model.
