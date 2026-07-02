# Boot universal section is a Static embed, not a hymns band

SL-186 delivered a **closed** hymns band registry (`src/hymns.rs`, INV-1,
STD-001 single source), declaration order fixed:

```
Preamble · Harness · Model · Role · Stage · Project
```

There is **no `universal` band**. `Preamble`/`Project` are the two bands with no
namesake axis, but `install/hymns/preamble/core.md` is worker-preamble prose
("You are a doctrine dispatch worker…") and `project` is empty — neither is a
generic "applies to every agent" bucket.

So when SL-187's design says a "`universal`-band hymns section" on the disk
`boot.md`, that is **loose wording, not a hymns-corpus band**. The delivered
vehicle is one **authored `SourceKind::Static` embed section** in
`boot_sequence()` (`src/boot.rs:104`) — the same pattern as
`SourceKind::Static("routing-process.md")` (body via `install::asset_text`, read
from the compiled embed, never from disk `.doctrine/`).

**Cache-ordering invariant** (`src/boot.rs:100-103`): the `ExecPath` ("Invoking
doctrine") section is deliberately **last** because it is build-volatile —
tailing it confines a path change to the snapshot tail and keeps the governance
prefix cache-warm. Stable authored sections (the model-band floor directive, the
inlined onboarding memories) belong in the **onboarding action-tail** — after
`Onboarding`, **before** `ExecPath` — never after it.

Corollary: harness/model/role/stage bands are stdout-only (`prompt resolve`),
**never** baked into the disk `boot.md`. The disk sector is universal +
model-agnostic by construction.

Surfaced phase-planning SL-187 PHASE-01 (design-vs-delivered mismatch, an RV-213
carry-over class). See SL-187 design §5.2 (reconciled).
