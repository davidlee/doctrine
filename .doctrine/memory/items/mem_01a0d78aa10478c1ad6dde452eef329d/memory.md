Two cumulative conditions bind the inquiry map: `initial-concerns-recorded` and
`user-accepts-sufficiency` — both `attested`, both `binding = inquiry-map`, both
`reach = cumulative` (`install/design-run-stages.md`). Cumulative reach means every
edge above their own re-derives them against current content.

**The asymmetry.** `NodeMaterial` (`src/design_run/inquiry.rs`) excludes lifecycle
and disposition, pinned by `node_material_ignores_progress_and_observes_shape`
(`src/design_run/tests.rs`). So:

- resolving, reopening, deferring or pruning a node does **not** invalidate them;
- adding or moving a node, or editing `needs` / `parent`, **does** — and the run
  then re-faces the human gates at `inquiring→drafting`, `drafting→reviewing` and
  `reviewing→locked`.

`blocking-inquiries-dispositioned` is derived and cumulative too, so a genuinely
new blocking question does still block advance. That part is correct.

**What to expect from this.** Runs front-load the whole map during exploring /
inquiring, get sufficiency accepted once, then never touch shape again. Do **not**
read a frozen map as an agent not knowing it could grow: `DEC-061` permits nodes
to be added as discoveries become concrete, `DEC-063` makes redeclaration the
edit verb, and there is **no stage gate at all** on `declare` (admissible even at
`locked`). `SL-258` and `SL-259` exercised mid-run growth. The disincentive is the
re-attestation cost, not a refusal.

**Tension worth knowing.** `DEC-062` says ordinary map maintenance — "adding,
moving, pinning, deferring, or pruning nodes" — "does not require human
approval". A shape change re-opens a human gate. Tracked as `IMP-469`; the
unwritten intent is `IMP-471`.