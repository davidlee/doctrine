# Notes SL-115: Decompose main.rs: relocate orphan runners, extract cli arg modules

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

---

## 2026-06-20 — Plan complete, gated on SL-129

### Done

- **Design** authored, committed, and confirmed through 3 codex adversarial rounds (design §4/§5/§6 → round 2; §4 fold → round 3; exhaustive sweep → round 4).
- **Plan** authored: `plan.toml` + `plan.md`. 4 phases, sequencing forced by reachability (shells out before the match follows).
- **Phase sheets** materialised under `.doctrine/state/slice/115/phases/` — all 4 phases have their TOML + MD sheets.

### Commit ladder

```
92f31ac9 design(SL-115): fold codex round 2 — D1a sideways-call carve-out + V7 parse-regression
79c55de3 design(SL-115): fold codex round 3 — SpecReq→spec (second cycle-former); scaffold plan
50d02ea7 design(SL-115): fold codex round 4 — exhaustive cycle sweep clean; complete plan
```

### Key findings from codex adversarial audit

**B4 (round 2, blocker):** D1 body-relocation mints kind→kind edges the sink proof never covered — `MemoryCommand::Sync` → corpus via `MemoryCommand::run_*`. Nominal routing misroutes close cycles.

**B6 (round 3, blocker):** `SpecReqCommand` → requirement.rs (second cycle-former). `run_req_*` lives in `spec.rs`, not `requirement.rs`.

**Round 4 (exhaustive, clean):** Full per-kind table, `MemoryCommand::Sync`→corpus is the sole cycle-former, no third. Two factual nits fixed (Export mislabel, catalog orphans).

### Root fix: D1a rule

Route a kind's dispatch to the module its `run_*` body actually calls, not the nominal kind name. Body relocation from inert `main` into a command-tier module mints new production edges; nominal misroutes close cycles.

Two known instances:
- `MemoryCommand::Sync` → corpus → stays in `commands/` sink shell (PHASE-04), NOT `memory.rs`
- `SpecReqCommand` → spec.rs (own-module, zero edge), NOT `requirement.rs`

### V7 strengthened

`--help` snapshot + parse-regression covering `conflicts_with`, `value_parser`, `value_delimiter`, `requires` — these catch what a `--help` diff cannot.

### Per-batch gate (PHASE-03)

Gate runs after EACH domain batch in PHASE-03. Any tangle growth past `[tangle_baseline] command = 120` HALTS the batch for restructure — never an auto-accept.

### Gate: SL-129 must land first

PHASE-01 EN-2 gates on SL-129 landing first (`after` edge, R1). SL-129 consolidates `entity::id_path`; relocating the now-thinner resolvers afterwards keeps its 93-site inventory valid. **Do not flip SL-115 to ready/execute until SL-129 is in.**

When SL-129 is in: `/phase-plan PHASE-01` → `/execute`.

### Verification gate

No code moved yet (design + plan phase only). `just check` has not changed since last design commit. PHASE-01 builds the proof harness before any relocation.

### Durable items for memory

- **D1a rule** (dispatch routing by actual body calls, not nominal kind) — should be recorded as a `pattern` (or `design`?) memory for the dispatch layer.
