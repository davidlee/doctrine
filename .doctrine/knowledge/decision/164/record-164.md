# DEC-164: Scope switch evicts the abandoned entry

## Decision

`R10` is engineered, not documented. When doctrine writes one Claude settings
scope it also **sweeps the sibling file**, dropping this spec's owned entries,
and **reports the eviction** in the installer target line [[DEC-163]] adds.

Drop-only: the sweep never inserts into the file it is leaving. It touches only
the two files doctrine itself writes — no filesystem walk, no user or managed
settings.

## Why not documentation

Both-scopes is never a configuration anyone chose. Scopes **merge** — `OQ-6`
settled that by probe — so a doctrine hook present in both files simply fires
twice. There is no legitimate state it represents; it is always the defect.

So the eviction is not a policy imposed on the user. The merge core already
guarantees *exactly one canonical doctrine entry per spec*, and that guarantee is
scoped to a single file only because doctrine has only ever written one file.
Once doctrine can write either of two, the invariant's natural domain is the
pair. This finishes an existing invariant rather than handling an edge case.

Documentation was rejected on three grounds:

- **The failure is silent.** A hook firing twice means memory-sync runs twice and
  boot emits twice — it presents as mild slowness and nothing else. Documentation
  works when the failure is noticeable; a note whose remedy nobody is prompted to
  apply will not be applied.
- **Nothing would report it.** The leg that would have surfaced the stale entry
  went to IMP-407 with the doctor.
- **It is the certain path, not an edge case.** The default flip from
  `settings.local.json` to `settings.json` double-fires the memory-sync hook for
  **every existing install**, this repo included, on the first run after the
  change ships.

Together those mean the documentation option knowingly ships a guaranteed,
silent, undetectable defect. SL-250's less-code posture defers to documentation
where a gotcha *can* be documented; it does not cover a defect the documentation
cannot realistically reach.

## Why it is cheap

Reuse only, no new machinery:

- `owned_positions` and `drop_owned_hooks` already exist (`src/boot.rs:1039`,
  used at `:1201`). The sweep is `plan_hook` with the insert step omitted.
- `install_hook_to_file` already takes its target file per call, so the sibling
  path is the same parameter with the other constant.
- Because [[DEC-161]] keeps ownership **command-only**, the sweep inherits the
  healing property: a stale-matcher entry in the abandoned file is still
  recognised as ours and still evicted.

## Safety

Eviction is gated by the same ownership predicate that protects foreign entries
during a normal merge, so nothing a user wrote is at risk. This matters because
SL-250's less-code posture explicitly does **not** relax the never-clobber
contract — refusing to destroy a user's own content is correctness, not
edge-case handling.

The reporting rider is what keeps a destructive-looking write honest: the
installer says what it removed and from where, without building any of IMP-407's
diagnostic leg.

## Consequence

Repairs the operational half of the reversibility condition [[DEC-162]] was
accepted under. Changing scope becomes: edit the key, run install, stale entries
swept — rather than a manual cleanup the user must know to perform.

Recorded from design run `dr-019fd692` checkpoint `cp-4` disposing `inq-5`.
